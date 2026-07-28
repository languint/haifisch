use crate::board::{
    bitboard::Bitboard, castling::CastlingRights, color::Color, r#move::Move, piece::Piece,
    square::Square, undo::Undo, zobrist::Zobrist,
};

pub mod bitboard;
pub mod castling;
pub mod color;
pub mod fen;
pub mod file;
pub mod r#move;
pub mod piece;
pub mod rank;
pub mod square;
pub mod undo;
pub mod zobrist;

/// Mailbox empty sentinel.
const MAILBOX_EMPTY: u8 = 0;

/// Engine game state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
    mailbox: [u8; 64],
    kings: [Square; 2],
    side_to_move: Color,
    castling: CastlingRights,
    ep_square: Option<Square>,
    halfmove_clock: u16,
    fullmove_number: u16,
    hash: u64,
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
            mailbox: [MAILBOX_EMPTY; 64],
            kings: [Square::A1, Square::A1],
            side_to_move: Color::White,
            castling: CastlingRights::NONE,
            ep_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            hash: 0,
        }
    }

    /// Starting position.
    #[must_use]
    pub fn startpos() -> Self {
        Self::from_fen(fen::STARTPOS_FEN).unwrap_or_default()
    }

    #[inline]
    #[must_use]
    pub const fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline]
    #[must_use]
    pub const fn castling(&self) -> CastlingRights {
        self.castling
    }

    #[inline]
    #[must_use]
    pub const fn ep_square(&self) -> Option<Square> {
        self.ep_square
    }

    #[inline]
    #[must_use]
    pub const fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    #[inline]
    #[must_use]
    pub const fn fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    #[inline]
    #[must_use]
    pub const fn hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    #[must_use]
    pub const fn king_square(&self, color: Color) -> Square {
        self.kings[color.to_u8() as usize]
    }

    /// Piece and color on `square`, if occupied.
    #[inline]
    #[must_use]
    pub const fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        decode_mailbox(self.mailbox[square.to_u8() as usize])
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

    /// Place a piece, replacing any occupant. Updates hash.
    pub fn set_piece(&mut self, piece: Piece, color: Color, square: Square) {
        self.clear_square(square);
        let mask = Bitboard::mask_from_square(square);
        self.pieces[piece.to_u8() as usize] |= mask;
        self.colors[color.to_u8() as usize] |= mask;
        self.mailbox[square.to_u8() as usize] = encode_mailbox(piece, color);
        if matches!(piece, Piece::King) {
            self.kings[color.to_u8() as usize] = square;
        }
        self.hash ^= Zobrist::piece(color, piece, square);
    }

    /// Remove any piece on `square`. Updates hash.
    pub fn clear_square(&mut self, square: Square) {
        let Some((piece, color)) = self.piece_at(square) else {
            return;
        };
        let mask = Bitboard::mask_from_square(square);
        self.pieces[piece.to_u8() as usize] &= !mask;
        self.colors[color.to_u8() as usize] &= !mask;
        self.mailbox[square.to_u8() as usize] = MAILBOX_EMPTY;
        self.hash ^= Zobrist::piece(color, piece, square);
    }

    /// Recompute Zobrist hash from current placement and state.
    pub fn recompute_hash(&mut self) {
        let mut hash = 0u64;
        for square in Square::ALL {
            if let Some((piece, color)) = self.piece_at(square) {
                hash ^= Zobrist::piece(color, piece, square);
            }
        }
        if self.side_to_move == Color::Black {
            hash ^= Zobrist::SIDE;
        }
        hash ^= Zobrist::castling(self.castling);
        if let Some(ep) = self.ep_square {
            hash ^= Zobrist::ep_file(ep.file());
        }
        self.hash = hash;
    }

    /// Apply `m` and return undo information.
    ///
    /// # Panics
    /// Debug-asserts that `from` is occupied. Calling with an illegal move is undefined.
    #[must_use]
    pub fn make_move(&mut self, m: Move) -> Undo {
        let us = self.side_to_move;
        let them = us.flip();
        let from = m.from();
        let to = m.to();

        let mut undo = Undo {
            captured: None,
            castling: self.castling,
            ep_square: self.ep_square,
            halfmove_clock: self.halfmove_clock,
            hash: self.hash,
        };

        // Strip EP from hash; may set a new one later.
        if let Some(ep) = self.ep_square {
            self.hash ^= Zobrist::ep_file(ep.file());
        }
        self.ep_square = None;

        let Some((moving_piece, _)) = self.piece_at(from) else {
            debug_assert!(false, "make_move: empty from-square");
            return undo;
        };

        let mut captured = None;

        self.hash ^= Zobrist::castling(self.castling);

        if m.is_castle() {
            self.clear_square(from);
            self.set_piece(Piece::King, us, to);
            let (rook_from, rook_to) = castle_rook_squares(us, to);
            self.clear_square(rook_from);
            self.set_piece(Piece::Rook, us, rook_to);
            self.castling.remove(castling_rights_for_color(us));
            self.halfmove_clock += 1;
        } else if m.is_en_passant() {
            let cap_sq = ep_captured_square(us, to);
            captured = Some(Piece::Pawn);
            self.clear_square(cap_sq);
            self.clear_square(from);
            self.set_piece(Piece::Pawn, us, to);
            self.halfmove_clock = 0;
        } else {
            if let Some((cap_piece, _)) = self.piece_at(to) {
                captured = Some(cap_piece);
                self.clear_square(to);
            }
            self.clear_square(from);
            let placed = m.promotion().map_or(moving_piece, |promo| promo);
            self.set_piece(placed, us, to);

            if matches!(moving_piece, Piece::Pawn) || captured.is_some() {
                self.halfmove_clock = 0;
            } else {
                self.halfmove_clock += 1;
            }

            if m.is_double_push() {
                let ep = ep_square_after_double(us, from);
                self.ep_square = Some(ep);
                self.hash ^= Zobrist::ep_file(ep.file());
            }
        }

        self.update_castling_rights(from, to, moving_piece, us, captured);
        self.hash ^= Zobrist::castling(self.castling);

        undo.captured = captured;

        self.side_to_move = them;
        self.hash ^= Zobrist::SIDE;

        if us == Color::Black {
            self.fullmove_number += 1;
        }

        undo
    }

    /// Reverse a previous [`Self::make_move`].
    pub fn unmake_move(&mut self, m: Move, undo: Undo) {
        let them = self.side_to_move;
        let us = them.flip();
        let from = m.from();
        let to = m.to();

        self.side_to_move = us;
        if us == Color::Black {
            self.fullmove_number -= 1;
        }

        self.hash = undo.hash;
        self.castling = undo.castling;
        self.ep_square = undo.ep_square;
        self.halfmove_clock = undo.halfmove_clock;

        if m.is_castle() {
            let (rook_from, rook_to) = castle_rook_squares(us, to);
            self.clear_square_raw(to);
            self.clear_square_raw(rook_to);
            self.put_piece_raw(Piece::King, us, from);
            self.put_piece_raw(Piece::Rook, us, rook_from);
            return;
        }

        let moved = if m.promotion().is_some() {
            Piece::Pawn
        } else if let Some((piece, _)) = self.piece_at(to) {
            piece
        } else {
            debug_assert!(false, "unmake_move: empty to-square");
            Piece::Pawn
        };

        self.clear_square_raw(to);
        self.put_piece_raw(moved, us, from);

        if m.is_en_passant() {
            let cap_sq = ep_captured_square(us, to);
            self.put_piece_raw(Piece::Pawn, them, cap_sq);
        } else if let Some(cap) = undo.captured {
            self.put_piece_raw(cap, them, to);
        }
    }

    fn update_castling_rights(
        &mut self,
        from: Square,
        to: Square,
        moving_piece: Piece,
        us: Color,
        captured: Option<Piece>,
    ) {
        if matches!(moving_piece, Piece::King) {
            self.castling.remove(castling_rights_for_color(us));
        }
        if matches!(moving_piece, Piece::Rook) {
            self.castling.remove(rook_right_for_square(from));
        }
        if captured == Some(Piece::Rook) {
            self.castling.remove(rook_right_for_square(to));
        }
        // Also clear if a rook is captured via EP? impossible.
        let _ = captured;
    }

    /// Clear without touching hash (used in unmake after hash restore).
    fn clear_square_raw(&mut self, square: Square) {
        let Some((piece, color)) = self.piece_at(square) else {
            return;
        };
        let mask = Bitboard::mask_from_square(square);
        self.pieces[piece.to_u8() as usize] &= !mask;
        self.colors[color.to_u8() as usize] &= !mask;
        self.mailbox[square.to_u8() as usize] = MAILBOX_EMPTY;
    }

    /// Place without touching hash (used in unmake after hash restore).
    fn put_piece_raw(&mut self, piece: Piece, color: Color, square: Square) {
        let mask = Bitboard::mask_from_square(square);
        self.pieces[piece.to_u8() as usize] |= mask;
        self.colors[color.to_u8() as usize] |= mask;
        self.mailbox[square.to_u8() as usize] = encode_mailbox(piece, color);
        if matches!(piece, Piece::King) {
            self.kings[color.to_u8() as usize] = square;
        }
    }
}

#[inline]
#[must_use]
const fn encode_mailbox(piece: Piece, color: Color) -> u8 {
    1 + color.to_u8() * 6 + piece.to_u8()
}

#[inline]
#[must_use]
const fn decode_mailbox(value: u8) -> Option<(Piece, Color)> {
    if value == MAILBOX_EMPTY {
        return None;
    }
    let value = value - 1;
    let piece = unsafe { Piece::from_u8_unchecked(value % 6) };
    let color = unsafe { Color::from_u8_unchecked(value / 6) };
    Some((piece, color))
}

#[inline]
const fn castling_rights_for_color(color: Color) -> CastlingRights {
    match color {
        Color::White => CastlingRights::WHITE_KING.union(CastlingRights::WHITE_QUEEN),
        Color::Black => CastlingRights::BLACK_KING.union(CastlingRights::BLACK_QUEEN),
    }
}

#[inline]
const fn rook_right_for_square(square: Square) -> CastlingRights {
    match square {
        Square::A1 => CastlingRights::WHITE_QUEEN,
        Square::H1 => CastlingRights::WHITE_KING,
        Square::A8 => CastlingRights::BLACK_QUEEN,
        Square::H8 => CastlingRights::BLACK_KING,
        _ => CastlingRights::NONE,
    }
}

/// Rook from/to squares given king destination after castling.
#[inline]
const fn castle_rook_squares(color: Color, king_to: Square) -> (Square, Square) {
    match (color, king_to) {
        (Color::White, Square::G1) => (Square::H1, Square::F1),
        (Color::White, Square::C1) => (Square::A1, Square::D1),
        (Color::Black, Square::G8) => (Square::H8, Square::F8),
        (Color::Black, Square::C8) => (Square::A8, Square::D8),
        _ => (Square::A1, Square::A1), // unreachable for legal castles
    }
}

#[inline]
const fn ep_captured_square(us: Color, to: Square) -> Square {
    let delta: i8 = match us {
        Color::White => -8,
        Color::Black => 8,
    };
    let sq = to.to_u8().cast_signed() + delta;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    unsafe {
        Square::from_u8_unchecked(sq as u8)
    }
}

#[inline]
const fn ep_square_after_double(us: Color, from: Square) -> Square {
    let delta: i8 = match us {
        Color::White => 8,
        Color::Black => -8,
    };
    let sq = from.to_u8().cast_signed() + delta;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    unsafe {
        Square::from_u8_unchecked(sq as u8)
    }
}
