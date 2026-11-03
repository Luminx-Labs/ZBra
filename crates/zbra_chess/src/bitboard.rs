//! Bitboard representation and operations.
//!
//! This module contains the definition of the [`BitBoard`] struct which is BitBoard.
//! All the implementation about it is defined here to use it.
//! But also other types like [`Piece`] and [`Sides`] to
//! represent the piece and sides efficiently

/// A 64-bit set of chessboard squares.
///
/// Bit `0` represents `A1`, bit `1` represents `B1`, and bit `63` represents
/// `H8`. The public tuple field exposes the raw bit pattern for evaluation and
/// engine integrations.
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

impl BitBoard {
    /// An empty bitboard.
    pub const EMPTY: Self = Self(0);

    /// Creates a bitboard containing one square.
    #[inline]
    pub const fn from_square(square: usize) -> Option<Self> {
        if square < 64 {
            Some(Self(1_u64 << square))
        } else {
            None
        }
    }

    /// Returns the number of occupied squares.
    #[inline]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Tests whether a square is occupied.
    #[inline]
    pub const fn contains(self, square: usize) -> bool {
        square < 64 && self.0 & (1_u64 << square) != 0
    }

    /// Removes and returns the least-significant occupied square.
    #[inline]
    pub fn pop_lsb(&mut self) -> Option<usize> {
        if self.0 == 0 {
            None
        } else {
            let square = self.0.trailing_zeros() as usize;
            self.0 &= self.0 - 1;
            Some(square)
        }
    }
}

impl std::ops::BitOr for BitBoard {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for BitBoard {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for BitBoard {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for BitBoard {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

// === Sides ===

/// Internal side-index constants used by the board implementation.
#[derive(Debug)]
pub(crate) struct Sides;

impl Sides {
    /// White side is also indicated by 0
    pub(crate) const WHITE: usize = 0;
}

// === Pieces ===

/// Internal piece-index constants used by the board implementation.
#[derive(Debug)]
pub(crate) struct Pieces;

impl Pieces {
    pub(crate) const KING: usize = 0;
    pub(crate) const QUEEN: usize = 1;
    pub(crate) const ROOK: usize = 2;
    pub(crate) const BISHOP: usize = 4;
    pub(crate) const KNIGHT: usize = 3;
    pub(crate) const PAWN: usize = 5;
}
