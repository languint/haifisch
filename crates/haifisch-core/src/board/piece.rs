#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub const ALL: [Self; 6] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];

    /// Create a [`Piece`] from a `u8`.
    ///
    /// # Safety
    /// `n` must be in `0`..=`5`, any other value will result in undefined behavior.
    #[inline]
    #[must_use]
    pub const unsafe fn from_u8_unchecked(n: u8) -> Self {
        debug_assert!(n < 6);
        unsafe { std::mem::transmute::<u8, Self>(n) }
    }

    /// Create a `u8` from a [`Piece`]
    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Get the algebraic char for a [`Piece`].
    #[inline]
    #[must_use]
    pub const fn to_char(self) -> Option<char> {
        match self {
            Self::Pawn => None,
            Self::Knight => Some('N'),
            Self::Bishop => Some('B'),
            Self::Rook => Some('R'),
            Self::Queen => Some('Q'),
            Self::King => Some('K'),
        }
    }
}
