use crate::board::{file::File, rank::Rank, square::Square};

/// A bitboard wrapper around a `u64` where each bit represents a [`Square`] on a chessboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const FULL: Self = Self(u64::MAX);
    pub const EMPTY: Self = Self(0);

    /// Create a [`Bitboard`] from a `u64`.
    #[inline]
    #[must_use]
    pub const fn from_u64(n: u64) -> Self {
        Self(n)
    }
}

impl Bitboard {
    /// Get the `u64` value of the [`Bitboard`]
    #[inline]
    #[must_use]
    pub const fn to_u64(self) -> u64 {
        self.0
    }
}

impl Bitboard {
    const FILE_MASKS: [u64; 8] = [
        0x0101_0101_0101_0101,
        0x0202_0202_0202_0202,
        0x0404_0404_0404_0404,
        0x0808_0808_0808_0808,
        0x1010_1010_1010_1010,
        0x2020_2020_2020_2020,
        0x4040_4040_4040_4040,
        0x8080_8080_8080_8080,
    ];

    const RANK_MASKS: [u64; 8] = [
        0x0000_0000_0000_00FF,
        0x0000_0000_0000_FF00,
        0x0000_0000_00FF_0000,
        0x0000_0000_FF00_0000,
        0x0000_00FF_0000_0000,
        0x0000_FF00_0000_0000,
        0x00FF_0000_0000_0000,
        0xFF00_0000_0000_0000,
    ];

    /// Create a [`Bitboard`] mask from a given [`Square`].
    #[inline]
    #[must_use]
    pub const fn mask_from_square(square: Square) -> Self {
        Self(1u64 << square.to_u8())
    }

    /// Create a [`Bitboard`] mask from a given [`File`].
    #[inline]
    #[must_use]
    pub const fn mask_from_file(file: File) -> Self {
        // O(1) lookup instead of computing
        Self(Self::FILE_MASKS[file.to_u8() as usize])
    }

    /// Create a [`Bitboard`] mask from a given [`Rank`].
    #[inline]
    #[must_use]
    pub const fn mask_from_rank(rank: Rank) -> Self {
        // O(1) lookup instead of computing
        Self(Self::RANK_MASKS[rank.to_u8() as usize])
    }

    /// Create a [`Bitboard`] mask from a square's parts.
    #[inline]
    #[must_use]
    pub const fn mask_from_parts(file: File, rank: Rank) -> Self {
        Self::mask_from_square(Square::from_parts(file, rank))
    }
}

impl Bitboard {
    /// Check if a given [`Square`] on the [`Bitboard`] is occupied
    #[inline]
    #[must_use]
    pub const fn contains(self, square: Square) -> bool {
        (self.0 & (1u64 << square.to_u8())) != 0
    }

    /// Returns `true` if no bits are set.
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Get the number of set bits in the [`Bitboard`]
    #[inline]
    #[must_use]
    pub const fn popcount(self) -> u32 {
        self.0.count_ones()
    }

    /// Index of the least-significant set bit, or `64` if empty.
    #[inline]
    #[must_use]
    pub const fn lsb(self) -> u32 {
        self.0.trailing_zeros()
    }

    /// Clear and return the least-significant set square, if any.
    #[inline]
    pub const fn pop_lsb(&mut self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }

        #[allow(clippy::cast_possible_truncation)]
        let sq = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1;
        Some(unsafe { Square::from_u8_unchecked(sq) })
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitXor for Bitboard {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl std::ops::Not for Bitboard {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{bitboard::Bitboard, file::File, rank::Rank, square::Square};

    #[test]
    fn bitboard_mask_from_file() {
        assert_eq!(
            Bitboard::mask_from_file(File::A),
            Bitboard::from_u64(0x0101_0101_0101_0101)
        );
        assert_eq!(
            Bitboard::mask_from_file(File::H),
            Bitboard::from_u64(0x8080_8080_8080_8080)
        );
    }

    #[test]
    fn bitboard_mask_from_rank() {
        assert_eq!(
            Bitboard::mask_from_rank(Rank::First),
            Bitboard::from_u64(0x0000_0000_0000_00FF)
        );
        assert_eq!(
            Bitboard::mask_from_rank(Rank::Eighth),
            Bitboard::from_u64(0xFF00_0000_0000_0000)
        );
    }

    #[test]
    fn bitboard_mask_from_square() {
        assert_eq!(
            Bitboard::mask_from_square(Square::A1),
            Bitboard::from_u64(1)
        );
        assert_eq!(
            Bitboard::mask_from_square(Square::H8),
            Bitboard::from_u64(1u64 << 63)
        );
    }

    #[test]
    fn bitboard_contains() {
        let bb = Bitboard::mask_from_square(Square::E4);

        assert!(bb.contains(Square::E4));
        assert!(!bb.contains(Square::E5));
    }

    #[test]
    fn bitboard_popcount() {
        let bb = Bitboard::mask_from_square(Square::A1)
            | Bitboard::mask_from_square(Square::H8)
            | Bitboard::mask_from_square(Square::E4);

        assert_eq!(bb.popcount(), 3);
    }

    #[test]
    fn bitboard_bitand() {
        let a = Bitboard::from_u64(0b1100);
        let b = Bitboard::from_u64(0b1010);

        assert_eq!(a & b, Bitboard::from_u64(0b1000));
    }

    #[test]
    fn bitboard_bitor() {
        let a = Bitboard::from_u64(0b1100);
        let b = Bitboard::from_u64(0b1010);

        assert_eq!(a | b, Bitboard::from_u64(0b1110));
    }

    #[test]
    fn bitboard_bitxor() {
        let a = Bitboard::from_u64(0b1100);
        let b = Bitboard::from_u64(0b1010);

        assert_eq!(a ^ b, Bitboard::from_u64(0b0110));
    }

    #[test]
    fn bitboard_not() {
        assert_eq!(!Bitboard::EMPTY, Bitboard::FULL,);

        assert_eq!(!Bitboard::FULL, Bitboard::EMPTY,);
    }

    #[test]
    fn bitboard_assign_ops() {
        let mut bb = Bitboard::from_u64(0b1100);

        bb &= Bitboard::from_u64(0b1010);
        assert_eq!(bb, Bitboard::from_u64(0b1000));

        bb |= Bitboard::from_u64(0b0011);
        assert_eq!(bb, Bitboard::from_u64(0b1011));

        bb ^= Bitboard::from_u64(0b1111);
        assert_eq!(bb, Bitboard::from_u64(0b0100));
    }
}
