use crate::board::{piece::Piece, square::Square};

/// A chess move.
///
/// # Memory Layout
/// | bits  | description |
/// | ----- | ----------- |
/// | 0-5   | from square |
/// | 6-11  | to square   |
/// | 12-15 | promotion   |
/// | 16-.. | flags       |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move(u32);

impl Move {
    #[must_use]
    pub const fn new(from: Square, to: Square) -> Self {
        Self((from.to_u8() as u32) | ((to.to_u8() as u32) << 6))
    }

    /// Create a promotion move. `promo` must be knight, bishop, rook, or queen.
    #[must_use]
    pub const fn new_promotion(from: Square, to: Square, promo: Piece) -> Self {
        let code = match promo {
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::Pawn | Piece::King => 0,
        };
        debug_assert!(code != 0);
        Self(Self::new(from, to).0 | (code << 12))
    }

    #[must_use]
    pub const fn from(self) -> Square {
        unsafe { Square::from_u8_unchecked((self.0 & 0x3F) as u8) }
    }

    #[must_use]
    pub const fn to(self) -> Square {
        unsafe { Square::from_u8_unchecked(((self.0 >> 6) & 0x3F) as u8) }
    }

    #[must_use]
    pub const fn promotion(self) -> Option<Piece> {
        match (self.0 >> 12) & 0xF {
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            _ => None,
        }
    }
}
