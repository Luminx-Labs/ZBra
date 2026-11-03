// === Castling ===

/// Provides labels for the [`CastlingRights`]
#[derive(Debug)]
pub(crate) struct Castling;
impl Castling {
    pub(crate) const NO_CASTLING: u8 = 0;
    pub(crate) const WHITE_00: u8 = 0b00000001;
    pub(crate) const WHITE_000: u8 = 0b00000010;
    pub(crate) const BLACK_00: u8 = 0b00000100;
    pub(crate) const BLACK_000: u8 = 0b00001000;

    pub(crate) const WHITE_CASTLING: u8 = Self::WHITE_00 | Self::WHITE_000;
    pub(crate) const BLACK_CASTLING: u8 = Self::BLACK_00 | Self::BLACK_000;
    pub(crate) const ANY_CASTLING: u8 = Self::BLACK_CASTLING | Self::WHITE_CASTLING;
}

// === Castling Rights ===

/// The four castling rights available in a position.
///
/// Rights are stored as an opaque bit mask. Use [`Self::contains`] and
/// [`Self::bits`] when integrating with a move generator or evaluator.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CastlingRights(u8);

impl CastlingRights {
    /// Returns a value with no castling rights.
    #[inline]
    pub const fn empty() -> Self {
        Self(Castling::NO_CASTLING)
    }
    /// Returns a value with all four castling rights enabled.
    #[inline]
    pub const fn all() -> Self {
        Self(Castling::ANY_CASTLING)
    }

    /// Returns the encoded castling rights.
    #[inline]
    pub const fn bits(&self) -> u8 {
        self.0
    }

    /// Returns whether all rights in `rights` are available.
    #[inline]
    pub const fn contains(&self, rights: u8) -> bool {
        self.0 & rights == rights
    }

    pub(crate) const fn from_bits(bits: u8) -> Self {
        Self(bits & Castling::ANY_CASTLING)
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self(Castling::ANY_CASTLING)
    }
}
