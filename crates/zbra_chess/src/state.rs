use crate::{rules::CastlingRights, square::Square};

/// Contains castling rights, move clocks, the en-passant square, and the side to move.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct State {
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
    half_move_counter: u8,
    full_move_number: u16,
    stm: usize,
}

impl State {
    /// Creates the state of a new game.
    #[inline]
    pub const fn new() -> Self {
        Self {
            castling_rights: CastlingRights::all(),
            en_passant_square: None,
            half_move_counter: 0,
            full_move_number: 1,
            stm: 0,
        }
    }

    /// Restores this state to the beginning of a game.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Returns the current castling rights.
    #[inline]
    pub const fn castling_rights(&self) -> &CastlingRights {
        &self.castling_rights
    }
    /// Returns the en-passant target square, if any.
    #[inline]
    pub const fn en_passant_square(&self) -> Option<Square> {
        self.en_passant_square
    }
    /// Returns the number of plies since the last pawn move or capture.
    #[inline]
    pub const fn half_move_counter(&self) -> u8 {
        self.half_move_counter
    }
    /// Returns the side to move (`0` for White, `1` for Black).
    #[inline]
    pub const fn side_to_move(&self) -> usize {
        self.stm
    }

    /// Returns the FEN full-move number.
    #[inline]
    pub const fn full_move_number(&self) -> u16 {
        self.full_move_number
    }

    pub(crate) fn advance(
        &mut self,
        castling_rights: CastlingRights,
        en_passant_square: Option<Square>,
        half_move_counter: u8,
    ) {
        self.castling_rights = castling_rights;
        self.en_passant_square = en_passant_square;
        self.half_move_counter = half_move_counter;
        if self.stm == 1 {
            self.full_move_number = self.full_move_number.saturating_add(1);
        }
        self.stm ^= 1;
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
