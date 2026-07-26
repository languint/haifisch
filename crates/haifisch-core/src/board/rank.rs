/// Represents a horizontal rank (1–8) on a chessboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Rank {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

impl Rank {
    /// Create a new [`Rank`] from a given `char`.
    ///
    /// # Safety
    /// `c` must be in '1'..='8', any other value will result in undefined behavior
    #[must_use]
    #[inline]
    pub const unsafe fn from_char_unchecked(c: char) -> Self {
        debug_assert!(matches!(c, '1'..='8'));
        unsafe { std::mem::transmute::<u8, Self>(c as u8 - b'1') }
    }

    /// Create a new [`Rank`] from a given `u8`.
    ///
    /// # Safety
    /// `n` must be in 0..=7, any other value will result in undefined behavior
    #[inline]
    #[must_use]
    pub const unsafe fn from_u8_unchecked(n: u8) -> Self {
        debug_assert!(n < 8);
        unsafe { std::mem::transmute::<u8, Self>(n) }
    }
}

impl Rank {
    /// Convert a [`Rank`] to a `u8`
    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Convert a [`Rank`] to a `char`
    #[inline]
    #[must_use]
    pub const fn to_char(self) -> char {
        (b'1' + self as u8) as char
    }
}

#[cfg(test)]
mod tests {
    use crate::board::rank::Rank;

    #[test]
    fn rank_from_char_unchecked() {
        unsafe {
            assert_eq!(Rank::from_char_unchecked('1'), Rank::First);
            assert_eq!(Rank::from_char_unchecked('8'), Rank::Eighth);
        }
    }

    #[test]
    fn rank_from_u8_unchecked() {
        unsafe {
            assert_eq!(Rank::from_u8_unchecked(0), Rank::First);
            assert_eq!(Rank::from_u8_unchecked(7), Rank::Eighth);
        }
    }

    #[test]
    fn rank_to_u8() {
        assert_eq!(Rank::First.to_u8(), 0);
        assert_eq!(Rank::Eighth.to_u8(), 7);
    }

    #[test]
    fn rank_to_char() {
        assert_eq!(Rank::First.to_char(), '1');
        assert_eq!(Rank::Eighth.to_char(), '8');
    }
}
