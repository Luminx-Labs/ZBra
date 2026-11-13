//! Fast, strict-enough PGN parsing with preservation of useful annotations.
//!
//! The parser accepts the PGN tag roster, SAN-like movetext, comments, NAGs,
//! nested annotation variations, and all four PGN result markers. It does
//! not claim that SAN is legal on a board; legality belongs to `zbra_chess`.

mod error;
mod parse;
mod types;

pub use error::ParserError;
pub use types::{Game, GameResult, Pgn, PlayerMove, Tag, parse_file};

/// Parses one or more PGN games from UTF-8 text.
pub fn parse(input: &str) -> Result<Pgn, ParserError> {
    parse::document(input)
}
