//! Parsers for chess interchange formats.
//!
//! This crate intentionally owns format parsing and has no dependency on the
//! chess engine. Parsed values can therefore be validated, stored, or adapted
//! to any board implementation.

use std::fmt;

/// A chess side as represented in a FEN position.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Color {
    /// White.
    White,
    /// Black.
    Black,
}

/// A piece kind as represented in a FEN position.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PieceKind {
    /// King.
    King,
    /// Queen.
    Queen,
    /// Rook.
    Rook,
    /// Bishop.
    Bishop,
    /// Knight.
    Knight,
    /// Pawn.
    Pawn,
}

/// A colored piece occupying one square in a parsed position.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Piece {
    /// Piece color.
    pub color: Color,
    /// Piece kind.
    pub kind: PieceKind,
}

/// A board coordinate encoded as a zero-based file and rank.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Square {
    /// Zero-based file (`a` is zero).
    pub file: u8,
    /// Zero-based rank (`1` is zero).
    pub rank: u8,
}

impl Square {
    fn parse(value: &str) -> Result<Self, FenError> {
        let bytes = value.as_bytes();
        if bytes.len() != 2
            || !(b'a'..=b'h').contains(&bytes[0])
            || !(b'1'..=b'8').contains(&bytes[1])
        {
            return Err(FenError::InvalidEnPassantSquare);
        }
        Ok(Self {
            file: bytes[0] - b'a',
            rank: bytes[1] - b'1',
        })
    }
}

/// A parsed, validated six-field FEN position.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FenPosition {
    /// Squares indexed by `rank * 8 + file`, from White's perspective.
    pub board: [Option<Piece>; 64],
    /// Side to move.
    pub side_to_move: Color,
    /// Castling rights in canonical `KQkq` order, or `-`.
    pub castling_rights: String,
    /// En-passant target square, if present.
    pub en_passant_square: Option<Square>,
    /// Half-move clock.
    pub halfmove_clock: u32,
    /// Full-move number.
    pub fullmove_number: u32,
}

/// Errors produced while parsing FEN.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FenError {
    /// FEN did not contain exactly six whitespace-separated fields.
    #[error("FEN must contain six fields")]
    InvalidFieldCount,
    /// The board description did not contain eight ranks.
    #[error("FEN board must contain eight ranks")]
    InvalidRankCount,
    /// A rank did not contain exactly eight squares.
    #[error("FEN rank does not contain eight squares")]
    InvalidRankWidth,
    /// A digit other than one through eight occurred in a rank.
    #[error("invalid empty-square count")]
    InvalidEmptySquareCount,
    /// An unknown piece symbol occurred.
    #[error("invalid FEN piece: {0}")]
    InvalidPiece(char),
    /// Each side must have exactly one king.
    #[error("FEN must contain one king per side")]
    InvalidKingCount,
    /// The active color was not `w` or `b`.
    #[error("invalid FEN side to move")]
    InvalidSideToMove,
    /// Castling rights were malformed.
    #[error("invalid FEN castling rights")]
    InvalidCastlingRights,
    /// An en-passant square was malformed.
    #[error("invalid FEN en-passant square")]
    InvalidEnPassantSquare,
    /// A numeric field was malformed.
    #[error("invalid FEN move counter")]
    InvalidMoveCounter,
    /// FEN full-move numbers start at one.
    #[error("FEN full-move number must be positive")]
    ZeroFullmoveNumber,
}

impl FenPosition {
    /// Parses and validates a complete six-field FEN string.
    pub fn parse(fen: &str) -> Result<Self, FenError> {
        let fields: Vec<_> = fen.split_whitespace().collect();
        if fields.len() != 6 {
            return Err(FenError::InvalidFieldCount);
        }
        let mut board = [None; 64];
        let ranks: Vec<_> = fields[0].split('/').collect();
        if ranks.len() != 8 {
            return Err(FenError::InvalidRankCount);
        }
        for (fen_rank, rank_text) in ranks.into_iter().enumerate() {
            let rank = 7 - fen_rank;
            let mut file = 0usize;
            for symbol in rank_text.chars() {
                if let Some(empty) = symbol.to_digit(10) {
                    if !(1..=8).contains(&empty) {
                        return Err(FenError::InvalidEmptySquareCount);
                    }
                    file += empty as usize;
                } else if let Some(piece) = parse_piece(symbol) {
                    if file >= 8 {
                        return Err(FenError::InvalidRankWidth);
                    }
                    board[rank * 8 + file] = Some(piece);
                    file += 1;
                } else {
                    return Err(FenError::InvalidPiece(symbol));
                }
                if file > 8 {
                    return Err(FenError::InvalidRankWidth);
                }
            }
            if file != 8 {
                return Err(FenError::InvalidRankWidth);
            }
        }
        if board
            .iter()
            .filter(|piece| {
                piece.is_some_and(|p| p.color == Color::White && p.kind == PieceKind::King)
            })
            .count()
            != 1
            || board
                .iter()
                .filter(|piece| {
                    piece.is_some_and(|p| p.color == Color::Black && p.kind == PieceKind::King)
                })
                .count()
                != 1
        {
            return Err(FenError::InvalidKingCount);
        }
        let side_to_move = match fields[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(FenError::InvalidSideToMove),
        };
        let castling_rights = parse_castling(fields[2])?;
        let en_passant_square = if fields[3] == "-" {
            None
        } else {
            Some(Square::parse(fields[3])?)
        };
        let halfmove_clock = fields[4]
            .parse()
            .map_err(|_| FenError::InvalidMoveCounter)?;
        let fullmove_number = fields[5]
            .parse()
            .map_err(|_| FenError::InvalidMoveCounter)?;
        if fullmove_number == 0 {
            return Err(FenError::ZeroFullmoveNumber);
        }
        Ok(Self {
            board,
            side_to_move,
            castling_rights,
            en_passant_square,
            halfmove_clock,
            fullmove_number,
        })
    }

    /// Serializes this parsed position into canonical six-field FEN.
    #[must_use]
    pub fn to_fen(&self) -> String {
        let mut placement = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                match self.board[rank * 8 + file] {
                    Some(piece) => {
                        if empty > 0 {
                            placement.push_str(&empty.to_string());
                            empty = 0;
                        }
                        placement.push(piece_symbol(piece));
                    }
                    None => empty += 1,
                }
            }
            if empty > 0 {
                placement.push_str(&empty.to_string());
            }
            if rank > 0 {
                placement.push('/');
            }
        }
        let side = match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        };
        let en_passant = self.en_passant_square.map_or_else(
            || "-".to_owned(),
            |square| format!("{}{}", char::from(b'a' + square.file), square.rank + 1),
        );
        format!(
            "{placement} {side} {} {en_passant} {} {}",
            self.castling_rights, self.halfmove_clock, self.fullmove_number
        )
    }
}

fn parse_piece(symbol: char) -> Option<Piece> {
    let kind = match symbol.to_ascii_lowercase() {
        'k' => PieceKind::King,
        'q' => PieceKind::Queen,
        'r' => PieceKind::Rook,
        'b' => PieceKind::Bishop,
        'n' => PieceKind::Knight,
        'p' => PieceKind::Pawn,
        _ => return None,
    };
    Some(Piece {
        color: if symbol.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        },
        kind,
    })
}

fn piece_symbol(piece: Piece) -> char {
    let symbol = match piece.kind {
        PieceKind::King => 'k',
        PieceKind::Queen => 'q',
        PieceKind::Rook => 'r',
        PieceKind::Bishop => 'b',
        PieceKind::Knight => 'n',
        PieceKind::Pawn => 'p',
    };
    if piece.color == Color::White {
        symbol.to_ascii_uppercase()
    } else {
        symbol
    }
}

fn parse_castling(value: &str) -> Result<String, FenError> {
    if value == "-" {
        return Ok("-".to_owned());
    }
    let mut rights = [false; 4];
    for symbol in value.chars() {
        let index = match symbol {
            'K' => 0,
            'Q' => 1,
            'k' => 2,
            'q' => 3,
            _ => return Err(FenError::InvalidCastlingRights),
        };
        if rights[index] {
            return Err(FenError::InvalidCastlingRights);
        }
        rights[index] = true;
    }
    Ok(['K', 'Q', 'k', 'q']
        .into_iter()
        .zip(rights)
        .filter_map(|(symbol, present)| present.then_some(symbol))
        .collect())
}

impl fmt::Display for FenPosition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_fen())
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, FenPosition};

    #[test]
    fn round_trips_fen() {
        let fen = "r3k2r/ppp2ppp/2n1b3/3qp3/3P4/2N1PN2/PPP2PPP/R2QK2R w KQkq - 4 9";
        assert_eq!(FenPosition::parse(fen).unwrap().to_fen(), fen);
    }

    #[test]
    fn rejects_short_board() {
        assert!(FenPosition::parse("8/8/8/8/8/8/8 w - - 0 1").is_err());
    }

    #[test]
    fn parses_side() {
        assert_eq!(
            FenPosition::parse("4k3/8/8/8/8/8/8/4K3 b - - 0 1")
                .unwrap()
                .side_to_move,
            Color::Black
        );
    }
}
