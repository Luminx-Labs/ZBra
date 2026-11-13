use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_till, take_until, take_while1},
    character::complete::{char, line_ending, multispace0, space0, space1},
    combinator::{all_consuming, map, opt},
    multi::{many0, many1},
    sequence::terminated,
};

use crate::{
    ParserError,
    types::{Game, GameResult, Pgn, PlayerMove, Tag},
};

type Movetext = (Vec<PlayerMove>, Vec<String>, Vec<String>, GameResult);

pub(crate) fn document(input: &str) -> Result<Pgn, ParserError> {
    match all_consuming(terminated(many1(game), multispace0)).parse(input) {
        Ok((_, games)) => Ok(Pgn::new(games)),
        Err(error) => Err(convert_error(input, error)),
    }
}

fn game(input: &str) -> IResult<&str, Game> {
    let (input, _) = multispace0(input)?;
    let (input, tags) = many0(terminated_tag).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, (moves, comments, variations, result)) = movetext(input)?;
    if tags.is_empty() && moves.is_empty() && comments.is_empty() && variations.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NonEmpty,
        )));
    }
    Ok((input, Game::new(tags, moves, comments, variations, result)))
}

fn terminated_tag(input: &str) -> IResult<&str, Tag> {
    let (input, _) = space0(input)?;
    let (input, _) = char('[').parse(input)?;
    let (input, _) = space0(input)?;
    let (input, name) = take_while1(|c: char| c.is_ascii_alphabetic() || c == '_').parse(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = quoted_string(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(']').parse(input)?;
    let (input, _) = opt(line_ending).parse(input)?;
    Ok((input, Tag::new(name.to_owned(), value)))
}

fn quoted_string(input: &str) -> IResult<&str, String> {
    let (mut rest, _) = char('"').parse(input)?;
    let mut value = String::new();
    loop {
        if let Some(ch) = rest.chars().next() {
            let size = ch.len_utf8();
            match ch {
                '"' => return Ok((rest.split_at(size).1, value)),
                '\\' => {
                    let escaped_input = rest.split_at(size).1;
                    let Some(escaped) = escaped_input.chars().next() else {
                        return Err(nom::Err::Incomplete(nom::Needed::new(1)));
                    };
                    value.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        _ => escaped,
                    });
                    rest = escaped_input.split_at(escaped.len_utf8()).1;
                }
                '\n' | '\r' => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        rest,
                        nom::error::ErrorKind::Escaped,
                    )));
                }
                _ => {
                    value.push(ch);
                    rest = rest.split_at(size).1;
                }
            }
        } else {
            return Err(nom::Err::Incomplete(nom::Needed::new(1)));
        }
    }
}

fn movetext(mut input: &str) -> IResult<&str, Movetext> {
    let mut moves = Vec::new();
    let mut comments = Vec::new();
    let mut variations = Vec::new();
    let mut result = None;
    let mut move_number = 1;
    let mut white_to_move = true;
    loop {
        let before = input;
        let (rest, _) = multispace0(input)?;
        input = rest;
        if input.is_empty() {
            break;
        }
        if input.starts_with('{') {
            let (rest, comment) = brace_comment(input)?;
            comments.push(comment);
            input = rest;
            continue;
        }
        if input.starts_with(';') {
            let (rest, comment) = line_comment(input)?;
            comments.push(comment);
            input = rest;
            continue;
        }
        if input.starts_with('(') {
            let (rest, variation) = variation(input)?;
            variations.push(variation);
            input = rest;
            continue;
        }
        if input.starts_with('[') {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Char,
            )));
        }
        if let Ok((rest, marker)) = result_token(input) {
            result = Some(marker);
            input = rest;
            break;
        }
        if let Ok((rest, number)) = move_number_token(input) {
            move_number = number;
            white_to_move = true;
            input = rest;
            continue;
        }
        let (rest, san) = san_token(input)?;
        let (rest, _) = multispace0(rest)?;
        let (rest, nags) = many0(nag).parse(rest)?;
        moves.push(PlayerMove::new(move_number, san, nags));
        if !white_to_move {
            move_number += 1;
        }
        white_to_move = !white_to_move;
        input = rest;
        if before.len() == input.len() {
            break;
        }
    }
    Ok((
        input,
        (
            moves,
            comments,
            variations,
            result.unwrap_or(GameResult::Ongoing),
        ),
    ))
}

fn move_number_token(input: &str) -> IResult<&str, u32> {
    let mut parser = map(take_while1(|c: char| c.is_ascii_digit()), |digits: &str| {
        digits.parse::<u32>().unwrap_or(u32::MAX)
    });
    let (input, number) = parser.parse(input)?;
    let (input, _) =
        if let Ok((rest, _)) = tag::<&str, &str, nom::error::Error<&str>>("...").parse(input) {
            (rest, ())
        } else {
            let (rest, _) = char('.').parse(input)?;
            (rest, ())
        };
    Ok((input, number))
}
fn san_token(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| !c.is_whitespace() && !"{}();".contains(c)),
        str::to_owned,
    )
    .parse(input)
}
fn result_token(input: &str) -> IResult<&str, GameResult> {
    let (rest, token) = take_while1(|c: char| !c.is_whitespace()).parse(input)?;
    match GameResult::parse(token) {
        Some(result) => Ok((rest, result)),
        None => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}
fn nag(input: &str) -> IResult<&str, String> {
    if input.starts_with('$') {
        map(
            take_while1(|c: char| c == '$' || c.is_ascii_digit()),
            str::to_owned,
        )
        .parse(input)
    } else {
        map(
            take_while1(|c: char| c == '?' || c == '!' || c == '+' || c == '#'),
            str::to_owned,
        )
        .parse(input)
    }
}
fn brace_comment(input: &str) -> IResult<&str, String> {
    let (input, _) = char('{').parse(input)?;
    let (rest, body) = take_until("}").parse(input)?;
    let (rest, _) = char('}').parse(rest)?;
    Ok((rest, body.trim().to_owned()))
}
fn line_comment(input: &str) -> IResult<&str, String> {
    let (input, _) = char(';').parse(input)?;
    let (rest, body) = take_till(|c| c == '\n' || c == '\r').parse(input)?;
    Ok((rest, body.trim().to_owned()))
}
fn variation(input: &str) -> IResult<&str, String> {
    let (input, _) = char('(').parse(input)?;
    let mut depth = 1_u32;
    let mut in_comment = false;
    let mut comment_line = false;
    for (offset, character) in input.char_indices() {
        if comment_line {
            if character == '\n' || character == '\r' {
                comment_line = false;
            }
            continue;
        }
        if in_comment {
            if character == '}' {
                in_comment = false;
            }
            continue;
        }
        match character {
            '{' => in_comment = true,
            ';' => comment_line = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let (body, rest) = input.split_at(offset);
                    let rest = rest.split_at(character.len_utf8()).1;
                    return Ok((rest, body.trim().to_owned()));
                }
            }
            _ => {}
        }
    }
    Err(nom::Err::Incomplete(nom::Needed::new(1)))
}

fn convert_error(input: &str, error: nom::Err<nom::error::Error<&str>>) -> ParserError {
    let (fragment, kind) = match error {
        nom::Err::Incomplete(_) => {
            return ParserError::Incomplete {
                offset: input.len(),
                context: "unfinished PGN construct",
            };
        }
        nom::Err::Error(e) | nom::Err::Failure(e) => (e.input, e.code),
    };
    ParserError::Invalid {
        offset: input.len() - fragment.len(),
        message: format!("{kind:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tags_comments_nags_and_result() {
        let pgn = document(
            "[Event \"Demo\"]\n[Site \"Local\\\"Room\"]\n\n1. e4 $1 e5 {reply} 2. Nf3 Nc6 1-0\n",
        )
        .unwrap();
        let game = &pgn.games()[0];
        assert_eq!(game.tags()[1].value(), "Local\"Room");
        assert_eq!(game.moves().len(), 4);
        assert_eq!(game.moves()[0].san(), "e4");
        assert_eq!(game.moves()[0].nags(), &["$1"]);
        assert_eq!(game.comments(), &["reply"]);
        assert_eq!(game.result(), GameResult::WhiteWin);
    }

    #[test]
    fn parses_multiple_games_and_variations() {
        let pgn = document("1. d4 d5 (1... Nf6 (1... d6)) 1/2-1/2\n\n[Result \"*\"]\n*\n").unwrap();
        assert_eq!(pgn.games().len(), 2);
        assert_eq!(pgn.games()[0].variations(), &["1... Nf6 (1... d6)"]);
        assert_eq!(pgn.games()[1].result(), GameResult::Ongoing);
    }

    #[test]
    fn rejects_unclosed_constructs() {
        assert!(document("[Event \"unfinished]\n").is_err());
        assert!(document("1. e4 {unfinished").is_err());
    }
}
