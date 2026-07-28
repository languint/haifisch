#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Create a [`Color`] from its `char`.
    ///
    /// # Safety
    /// `c` must be `'w'` or `'b'`, other values are undefined behavior.
    #[inline]
    #[must_use]
    pub const unsafe fn from_char_unchecked(c: char) -> Self {
        debug_assert!(matches!(c, 'w' | 'b'));

        unsafe { Self::from_u8_unchecked((c == 'b') as u8) }
    }

    /// Create a [`Color`] from a `u8`.
    ///
    /// # Safety
    /// `n` must be `0` or `1`, other values are undefined behavior.
    #[inline]
    #[must_use]
    pub const unsafe fn from_u8_unchecked(n: u8) -> Self {
        debug_assert!(n < 2);
        unsafe { std::mem::transmute(n) }
    }

    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    #[inline]
    #[must_use]
    pub const fn to_char(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }

    /// Opposite color.
    #[inline]
    #[must_use]
    pub const fn flip(self) -> Self {
        unsafe { Self::from_u8_unchecked(self.to_u8() ^ 1) }
    }
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        self.flip()
    }
}

#[cfg(test)]
mod tests {
    use crate::board::color::Color;

    #[test]
    fn not() {
        assert_eq!(!Color::White, Color::Black);
        assert_eq!(!Color::Black, Color::White);
    }

    #[test]
    fn from_char_unchecked() {
        unsafe {
            assert_eq!(Color::from_char_unchecked('w'), Color::White);
            assert_eq!(Color::from_char_unchecked('b'), Color::Black);
        }
    }

    #[test]
    fn double_not() {
        for color in [Color::White, Color::Black] {
            assert_eq!(!(!color), color);
        }
    }
}
