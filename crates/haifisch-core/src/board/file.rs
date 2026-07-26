/// Represents a vertical file (a–h) on a chessboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl File {
    /// Create a new [`File`] from a given `char`.
    ///
    /// # Safety
    /// `c` must be in 'a'..='h', any other value will result in undefined behavior
    #[inline]
    #[must_use]
    pub const unsafe fn from_char_unchecked(c: char) -> Self {
        debug_assert!(matches!(c, 'a'..='h'));
        unsafe { std::mem::transmute::<u8, Self>(c as u8 - b'a') }
    }

    /// Create a new [`File`] from a given `u8`.
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

impl File {
    /// Convert a [`File`] to a `u8`
    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Convert a [`File`] to a `char`
    #[inline]
    #[must_use]
    pub const fn to_char(self) -> char {
        (b'a' + self as u8) as char
    }
}

#[cfg(test)]
mod tests {
    use crate::board::file::File;

    #[test]
    fn file_from_char_unchecked() {
        unsafe {
            assert_eq!(File::from_char_unchecked('a'), File::A);
            assert_eq!(File::from_char_unchecked('h'), File::H);
        }
    }

    #[test]
    fn file_from_u8_unchecked() {
        unsafe {
            assert_eq!(File::from_u8_unchecked(0), File::A);
            assert_eq!(File::from_u8_unchecked(7), File::H);
        }
    }

    #[test]
    fn file_to_u8() {
        assert_eq!(File::A.to_u8(), 0);
        assert_eq!(File::H.to_u8(), 7);
    }

    #[test]
    fn file_to_char() {
        assert_eq!(File::A.to_char(), 'a');
        assert_eq!(File::H.to_char(), 'h');
    }
}
