use crate::board::{castling::CastlingRights, piece::Piece, square::Square};

/// Information needed to reverse a [`super::r#move::Move`].
#[derive(Debug, Clone, Copy)]
pub struct Undo {
    pub captured: Option<Piece>,
    pub castling: CastlingRights,
    pub ep_square: Option<Square>,
    pub halfmove_clock: u16,
    pub hash: u64,
}
