pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

use crate::board::{
    Board,
    bitboard::Bitboard,
    color::Color,
    piece::Piece,
    r#move::Move,
    rank::Rank,
    square::Square,
};
use crate::movegen::{
    bishop::attacks as bishop_attacks,
    king::KING_ATTACKS,
    knight::KNIGHT_ATTACKS,
    pawn::PAWN_ATTACKS,
    queen::attacks as queen_attacks,
    rook::attacks as rook_attacks,
};

/// Maximum number of pseudo-legal moves in any position.
pub const MAX_MOVES: usize = 256;

/// Stack-allocated list of moves.
#[derive(Debug, Clone)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl MoveList {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            moves: [Move::new(Square::A1, Square::A1); MAX_MOVES],
            len: 0,
        }
    }

    #[inline]
    pub const fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn push(&mut self, m: Move) {
        debug_assert!(self.len < MAX_MOVES);
        self.moves[self.len] = m;
        self.len += 1;
    }

    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate pseudo-legal moves for `color` into `moves`.
///
/// Moves that leave the king in check are included. Castling and en passant
/// are not generated yet (board state does not track them).
pub fn generate_pseudo_legal(board: &Board, color: Color, moves: &mut MoveList) {
    moves.clear();

    let friendly = board.color_bb(color);
    let enemy = board.color_bb(!color);
    let occupancy = board.occupancy();
    let empty = board.empty();

    generate_pawn_moves(board, color, enemy, empty, moves);
    generate_leaper_moves(
        board.pieces_of(Piece::Knight, color),
        &KNIGHT_ATTACKS,
        friendly,
        moves,
    );
    generate_slider_moves(
        board.pieces_of(Piece::Bishop, color),
        bishop_attacks,
        friendly,
        occupancy,
        moves,
    );
    generate_slider_moves(
        board.pieces_of(Piece::Rook, color),
        rook_attacks,
        friendly,
        occupancy,
        moves,
    );
    generate_slider_moves(
        board.pieces_of(Piece::Queen, color),
        queen_attacks,
        friendly,
        occupancy,
        moves,
    );
    generate_leaper_moves(
        board.pieces_of(Piece::King, color),
        &KING_ATTACKS,
        friendly,
        moves,
    );
}

fn generate_leaper_moves(
    mut pieces: Bitboard,
    table: &[Bitboard; 64],
    friendly: Bitboard,
    moves: &mut MoveList,
) {
    while let Some(from) = pieces.pop_lsb() {
        let mut targets = table[from.to_u8() as usize] & !friendly;
        while let Some(to) = targets.pop_lsb() {
            moves.push(Move::new(from, to));
        }
    }
}

fn generate_slider_moves(
    mut pieces: Bitboard,
    attacks: fn(Square, Bitboard) -> Bitboard,
    friendly: Bitboard,
    occupancy: Bitboard,
    moves: &mut MoveList,
) {
    while let Some(from) = pieces.pop_lsb() {
        let mut targets = attacks(from, occupancy) & !friendly;
        while let Some(to) = targets.pop_lsb() {
            moves.push(Move::new(from, to));
        }
    }
}

fn generate_pawn_moves(
    board: &Board,
    color: Color,
    enemy: Bitboard,
    empty: Bitboard,
    moves: &mut MoveList,
) {
    let mut pawns = board.pieces_of(Piece::Pawn, color);
    let (push_delta, double_rank, promo_rank): (i8, Rank, Rank) = match color {
        Color::White => (8, Rank::Second, Rank::Eighth),
        Color::Black => (-8, Rank::Seventh, Rank::First),
    };

    while let Some(from) = pawns.pop_lsb() {
        let from_i = from.to_u8().cast_signed();

        // Single push
        let single = from_i + push_delta;
        if (0..64).contains(&single) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let to = unsafe { Square::from_u8_unchecked(single as u8) };
            if empty.contains(to) {
                push_pawn_move(moves, from, to, promo_rank);

                // Double push
                if from.rank() == double_rank {
                    let double = from_i + push_delta * 2;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let to2 = unsafe { Square::from_u8_unchecked(double as u8) };
                    if empty.contains(to2) {
                        moves.push(Move::new(from, to2));
                    }
                }
            }
        }

        // Captures
        let mut attacks = PAWN_ATTACKS[color.to_u8() as usize][from.to_u8() as usize] & enemy;
        while let Some(to) = attacks.pop_lsb() {
            push_pawn_move(moves, from, to, promo_rank);
        }
    }
}

fn push_pawn_move(moves: &mut MoveList, from: Square, to: Square, promo_rank: Rank) {
    if to.rank() == promo_rank {
        moves.push(Move::new_promotion(from, to, Piece::Queen));
        moves.push(Move::new_promotion(from, to, Piece::Rook));
        moves.push(Move::new_promotion(from, to, Piece::Bishop));
        moves.push(Move::new_promotion(from, to, Piece::Knight));
    } else {
        moves.push(Move::new(from, to));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::file::File;
    use crate::movegen::{bishop, king, knight, pawn, queen, rook};

    #[test]
    fn knight_corner_attacks() {
        assert_eq!(knight::KNIGHT_ATTACKS[Square::A1 as usize].popcount(), 2);
        assert_eq!(knight::KNIGHT_ATTACKS[Square::E4 as usize].popcount(), 8);
    }

    #[test]
    fn king_center_attacks() {
        assert_eq!(king::KING_ATTACKS[Square::E4 as usize].popcount(), 8);
        assert_eq!(king::KING_ATTACKS[Square::A1 as usize].popcount(), 3);
    }

    #[test]
    fn pawn_attacks_do_not_wrap_files() {
        let a2 = Square::from_parts(File::A, Rank::Second);
        let white = pawn::PAWN_ATTACKS[Color::White.to_u8() as usize][a2.to_u8() as usize];
        assert!(white.contains(Square::from_parts(File::B, Rank::Third)));
        assert_eq!(white.popcount(), 1);
    }

    #[test]
    fn slider_empty_matches_table() {
        for sq in Square::ALL {
            let i = sq.to_u8() as usize;
            assert_eq!(bishop::attacks(sq, Bitboard::EMPTY), bishop::BISHOP_ATTACKS[i]);
            assert_eq!(rook::attacks(sq, Bitboard::EMPTY), rook::ROOK_ATTACKS[i]);
            assert_eq!(queen::attacks(sq, Bitboard::EMPTY), queen::QUEEN_ATTACKS[i]);
        }
    }

    #[test]
    fn rook_blocked_by_occupancy() {
        let occ = Bitboard::mask_from_square(Square::E6);
        let attacks = rook::attacks(Square::E4, occ);
        assert!(attacks.contains(Square::E6));
        assert!(!attacks.contains(Square::E7));
        assert!(attacks.contains(Square::E5));
        assert!(attacks.contains(Square::E1));
    }

    #[test]
    fn generate_knights_from_start_like_position() {
        let mut board = Board::new();
        board.place(Piece::Knight, Color::White, Square::B1);
        board.place(Piece::Knight, Color::White, Square::G1);
        // Block some squares with friendly pieces so we only get quiet targets
        board.place(Piece::Pawn, Color::White, Square::A3);
        board.place(Piece::Pawn, Color::White, Square::C3);
        board.place(Piece::Pawn, Color::White, Square::F3);
        board.place(Piece::Pawn, Color::White, Square::H3);

        let mut moves = MoveList::new();
        generate_pseudo_legal(&board, Color::White, &mut moves);

        // B1: D2 only (A3/C3 friendly); G1: E2 only (F3/H3 friendly)
        // Plus pawn pushes from A3,C3,F3,H3
        assert!(!moves.is_empty());
        assert!(moves.as_slice().contains(&Move::new(Square::B1, Square::D2)));
        assert!(moves.as_slice().contains(&Move::new(Square::G1, Square::E2)));
    }
}
