use crate::board::{bitboard::Bitboard, color::Color, piece::Piece, square::Square};

pub mod bitboard;
pub mod color;
pub mod file;
pub mod r#move;
pub mod piece;
pub mod rank;
pub mod square;

pub struct Board {
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
}

impl Default for Board {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    /// Create a board with no pieces.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pieces: [Bitboard::EMPTY; 6],
            colors: [Bitboard::EMPTY; 2],
        }
    }

    /// Place a piece on `square`. Does not clear an existing occupant.
    #[inline]
    pub fn place(&mut self, piece: Piece, color: Color, square: Square) {
        let mask = Bitboard::mask_from_square(square);
        self.pieces[piece.to_u8() as usize] |= mask;
        self.colors[color.to_u8() as usize] |= mask;
    }

    /// Get the [`Bitboard`] of a given [`Piece`].
    #[inline]
    #[must_use]
    pub const fn piece_bb(&self, piece: Piece) -> Bitboard {
        self.pieces[piece.to_u8() as usize]
    }

    /// Get the [`Bitboard`] of a given [`Color`].
    #[inline]
    #[must_use]
    pub const fn color_bb(&self, color: Color) -> Bitboard {
        self.colors[color.to_u8() as usize]
    }

    #[inline]
    #[must_use]
    pub const fn pieces_of(&self, piece: Piece, color: Color) -> Bitboard {
        Bitboard::from_u64(self.piece_bb(piece).to_u64() & self.color_bb(color).to_u64())
    }

    #[inline]
    #[must_use]
    pub const fn occupancy(&self) -> Bitboard {
        Bitboard::from_u64(self.colors[0].to_u64() | self.colors[1].to_u64())
    }

    #[inline]
    #[must_use]
    pub const fn empty(&self) -> Bitboard {
        Bitboard::from_u64(!self.occupancy().to_u64())
    }
}
