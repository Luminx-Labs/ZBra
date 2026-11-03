//! Strongly typed chess-domain values used by the public API.
//!
//! This module contains the following types:
//! - [`Color`]: Represents Color either White or Black.
//! - [`PieceKind`]: Represents Kind of Pieces from King to Pawn.
//! - [`Piece`]: Represents the actual piece in the Game with its color and kind.

/// A chess side.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Color {
    /// The side that moves from rank one toward rank eight.
    White = 0,
    /// The side that moves from rank eight toward rank one.
    Black = 1,
}

impl Color {
    /// Returns the array/bitboard index for this color.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns the opposing color.
    #[inline]
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

/// A piece kind independent of color.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PieceKind {
    /// Represents King in the Game
    King = 0,
    /// Represents Queen in the Game
    Queen = 1,
    /// Represents Rook in the Game
    Rook = 2,
    /// Represents Knight in the Game
    Knight = 3,
    /// Represents Bishop in the Game
    Bishop = 4,
    /// Represents Pawn in the Game
    Pawn = 5,
}

impl PieceKind {
    /// Returns the array/bitboard index for this piece kind.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns whether this piece kind is a legal promotion result.
    #[inline]
    pub const fn is_promotion(self) -> bool {
        matches!(self, Self::Queen | Self::Rook | Self::Bishop | Self::Knight)
    }
}

/// A colored piece occupying a square.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Piece {
    /// The piece's color.
    pub color: Color,
    /// The piece's kind.
    pub kind: PieceKind,
}
