use crate::board::{file::File, rank::Rank};

/// Represents a square on a chessboard.
/// 0 -> `Square::A1`
/// 63 -> `Square::H8`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    /// Creates a [`Square`] from an algebraic coordinate (e.g. `"e4"`).
    ///
    /// # Safety
    /// `s` must be 2 ASCII bytes long, `s[0]` must be in `a`..=`h`, and `s[1]` must be in `1`..=`8`.
    #[inline]
    #[must_use]
    pub const unsafe fn from_str_unchecked(s: &str) -> Self {
        debug_assert!(s.len() == 2);
        let s = s.as_bytes();
        let file_char = s[0];
        let rank_char = s[1];
        debug_assert!(matches!(file_char, b'a'..=b'h'));
        debug_assert!(matches!(rank_char, b'1'..=b'8'));

        unsafe { std::mem::transmute::<u8, Self>(file_char - b'a' + (rank_char - b'1') * 8) }
    }

    /// Create a new [`Square`] from its parts.
    #[inline]
    #[must_use]
    pub const fn from_parts(file: File, rank: Rank) -> Self {
        // SAFETY: guaranteed safe since Square is #[repr(u8)]
        unsafe { std::mem::transmute::<u8, Self>(file.to_u8() + rank.to_u8() * 8) }
    }

    /// Create a new [`Square`] from a `u8`.
    ///
    /// # Safety
    /// `n` must be < 64, any other value is undefined behavior.
    #[inline]
    #[must_use]
    pub const unsafe fn from_u8_unchecked(n: u8) -> Self {
        debug_assert!(n < 64);
        // SAFETY: safe if u8 < 64
        unsafe { std::mem::transmute::<u8, Self>(n) }
    }

    /// Convert a [`Square`] to a `u8`
    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

impl Square {
    /// Get the [`File`] of a [`Square`]
    #[inline]
    #[must_use]
    pub const fn file(self) -> File {
        unsafe { File::from_u8_unchecked((self as u8) & 7) }
    }

    /// Get the [`Rank`] of a [`Square`]
    #[inline]
    #[must_use]
    pub const fn rank(self) -> Rank {
        unsafe { Rank::from_u8_unchecked((self as u8) >> 3) }
    }

    /// Get the [`File`] and [`Rank`] of a [`Square`]
    #[inline]
    #[must_use]
    pub const fn to_parts(self) -> (File, Rank) {
        (self.file(), self.rank())
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{file::File, rank::Rank, square::Square};

    #[test]
    fn square_from_str_unchecked() {
        unsafe {
            assert_eq!(Square::from_str_unchecked("a1"), Square::A1);
            assert_eq!(Square::from_str_unchecked("h8"), Square::H8);
        }
    }

    #[test]
    fn square_from_parts() {
        assert_eq!(Square::from_parts(File::A, Rank::First), Square::A1);
        assert_eq!(Square::from_parts(File::H, Rank::Eighth), Square::H8);
    }

    #[test]
    fn square_from_u8() {
        unsafe {
            assert_eq!(Square::from_u8_unchecked(0), Square::A1);
            assert_eq!(Square::from_u8_unchecked(63), Square::H8);
        }
    }

    #[test]
    fn square_to_parts() {
        assert_eq!(Square::A1.to_parts(), (File::A, Rank::First));
        assert_eq!(Square::H8.to_parts(), (File::H, Rank::Eighth));
    }
}
