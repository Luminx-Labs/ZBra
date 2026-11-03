/// Represents a single square on the board.
/// # Representation
/// 1 is A1
/// 2 is B1
/// 64 is H8
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Square(usize);

impl Square {
    /// Creates a square from its zero-based bitboard index.
    #[inline]
    pub const fn from_index(index: usize) -> Option<Self> {
        if index < 64 { Some(Self(index)) } else { None }
    }

    pub(crate) const fn from_valid_index(index: usize) -> Self {
        Self(index)
    }

    /// Creates a square from its human-readable number (`A1` is 1 and `H8` is 64).
    #[inline]
    pub const fn new(number: usize) -> Option<Self> {
        if number > 0 && number <= 64 {
            Some(Self(number - 1))
        } else {
            None
        }
    }

    /// Returns the zero-based bitboard index of this square.
    #[inline]
    pub const fn index(self) -> usize {
        self.0
    }

    /// Creates a square from zero-based file and rank coordinates.
    #[inline]
    pub const fn from_file_rank(file: usize, rank: usize) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Self(rank * 8 + file))
        } else {
            None
        }
    }

    /// Returns the zero-based file (`A` is 0) and rank (`1` is 0).
    #[inline]
    pub const fn file_rank(self) -> (usize, usize) {
        (self.0 % 8, self.0 / 8)
    }
}

/// Labels for every [`Square`] on the board.
#[repr(usize)]
#[expect(missing_docs, reason = "`SquareLabels` is for convenience and represent all the square in the standard chess board")]
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SquareLabels {
    None,
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}
