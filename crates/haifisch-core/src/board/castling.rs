/// Castling rights packed into a `u8`.
///
/// | bit | right        |
/// | --- | ------------ |
/// | 0   | white king   |
/// | 1   | white queen  |
/// | 2   | black king   |
/// | 3   | black queen  |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const NONE: Self = Self(0);
    pub const WHITE_KING: Self = Self(0b0001);
    pub const WHITE_QUEEN: Self = Self(0b0010);
    pub const BLACK_KING: Self = Self(0b0100);
    pub const BLACK_QUEEN: Self = Self(0b1000);
    pub const ALL: Self = Self(0b1111);

    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & 0b1111)
    }

    #[inline]
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[inline]
    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[inline]
    pub const fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline]
    pub const fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

#[cfg(test)]
mod tests {
    use super::CastlingRights;

    #[test]
    fn union_and_contains() {
        let rights = CastlingRights::WHITE_KING.union(CastlingRights::BLACK_QUEEN);
        assert!(rights.contains(CastlingRights::WHITE_KING));
        assert!(rights.contains(CastlingRights::BLACK_QUEEN));
        assert!(!rights.contains(CastlingRights::WHITE_QUEEN));
    }
}
