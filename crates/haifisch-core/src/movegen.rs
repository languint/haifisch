pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

use crate::board::{
    Board,
    bitboard::Bitboard,
    castling::CastlingRights,
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

/// Returns `true` if `by` attacks `square`.
#[must_use]
pub fn is_square_attacked(board: &Board, square: Square, by: Color) -> bool {
    let occupancy = board.occupancy();

    let pawns = board.pieces_of(Piece::Pawn, by);
    if !(PAWN_ATTACKS[by.flip().to_u8() as usize][square.to_u8() as usize] & pawns).is_empty() {
        return true;
    }

    let knights = board.pieces_of(Piece::Knight, by);
    if !(KNIGHT_ATTACKS[square.to_u8() as usize] & knights).is_empty() {
        return true;
    }

    let kings = board.pieces_of(Piece::King, by);
    if !(KING_ATTACKS[square.to_u8() as usize] & kings).is_empty() {
        return true;
    }

    let bishops_queens =
        board.pieces_of(Piece::Bishop, by) | board.pieces_of(Piece::Queen, by);
    if !(bishop_attacks(square, occupancy) & bishops_queens).is_empty() {
        return true;
    }

    let rooks_queens = board.pieces_of(Piece::Rook, by) | board.pieces_of(Piece::Queen, by);
    if !(rook_attacks(square, occupancy) & rooks_queens).is_empty() {
        return true;
    }

    false
}

/// Returns `true` if `color`'s king is in check.
#[inline]
#[must_use]
pub fn in_check(board: &Board, color: Color) -> bool {
    is_square_attacked(board, board.king_square(color), color.flip())
}

/// Generate pseudo-legal moves for the side to move into `moves`.
///
/// Moves that leave the king in check may be included. Castling is only
/// emitted when the path is empty and the king does not castle through check.
pub fn generate_pseudo_legal(board: &Board, moves: &mut MoveList) {
    moves.clear();

    let color = board.side_to_move();
    let friendly = board.color_bb(color);
    let enemy = board.color_bb(color.flip());
    let occupancy = board.occupancy();
    let empty = board.empty();

    generate_pawn_moves(board, color, enemy, empty, moves);
    generate_leaper_moves(
        board.pieces_of(Piece::Knight, color),
        &KNIGHT_ATTACKS,
        friendly,
        enemy,
        moves,
    );
    generate_slider_moves(
        board.pieces_of(Piece::Bishop, color),
        bishop_attacks,
        friendly,
        enemy,
        occupancy,
        moves,
    );
    generate_slider_moves(
        board.pieces_of(Piece::Rook, color),
        rook_attacks,
        friendly,
        enemy,
        occupancy,
        moves,
    );
    generate_slider_moves(
        board.pieces_of(Piece::Queen, color),
        queen_attacks,
        friendly,
        enemy,
        occupancy,
        moves,
    );
    generate_leaper_moves(
        board.pieces_of(Piece::King, color),
        &KING_ATTACKS,
        friendly,
        enemy,
        moves,
    );
    generate_castling(board, color, empty, moves);
}

/// Generate legal moves for the side to move into `moves`.
pub fn generate_legal(board: &mut Board, moves: &mut MoveList) {
    let mut pseudo = MoveList::new();
    generate_pseudo_legal(board, &mut pseudo);
    moves.clear();

    let us = board.side_to_move();
    for &m in pseudo.as_slice() {
        let undo = board.make_move(m);
        if !in_check(board, us) {
            moves.push(m);
        }
        board.unmake_move(m, undo);
    }
}

fn generate_leaper_moves(
    mut pieces: Bitboard,
    table: &[Bitboard; 64],
    friendly: Bitboard,
    enemy: Bitboard,
    moves: &mut MoveList,
) {
    while let Some(from) = pieces.pop_lsb() {
        let mut targets = table[from.to_u8() as usize] & !friendly;
        while let Some(to) = targets.pop_lsb() {
            let flags = if enemy.contains(to) {
                Move::CAPTURE
            } else {
                0
            };
            moves.push(Move::new(from, to).with_flags(flags));
        }
    }
}

fn generate_slider_moves(
    mut pieces: Bitboard,
    attacks: fn(Square, Bitboard) -> Bitboard,
    friendly: Bitboard,
    enemy: Bitboard,
    occupancy: Bitboard,
    moves: &mut MoveList,
) {
    while let Some(from) = pieces.pop_lsb() {
        let mut targets = attacks(from, occupancy) & !friendly;
        while let Some(to) = targets.pop_lsb() {
            let flags = if enemy.contains(to) {
                Move::CAPTURE
            } else {
                0
            };
            moves.push(Move::new(from, to).with_flags(flags));
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

        let single = from_i + push_delta;
        if (0..64).contains(&single) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let to = unsafe { Square::from_u8_unchecked(single as u8) };
            if empty.contains(to) {
                push_pawn_move(moves, from, to, promo_rank, 0);

                if from.rank() == double_rank {
                    let double = from_i + push_delta * 2;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let to2 = unsafe { Square::from_u8_unchecked(double as u8) };
                    if empty.contains(to2) {
                        moves.push(Move::new(from, to2).with_flags(Move::DOUBLE_PUSH));
                    }
                }
            }
        }

        let mut attacks = PAWN_ATTACKS[color.to_u8() as usize][from.to_u8() as usize] & enemy;
        while let Some(to) = attacks.pop_lsb() {
            push_pawn_move(moves, from, to, promo_rank, Move::CAPTURE);
        }
    }

    if let Some(ep) = board.ep_square() {
        // Squares that can capture onto `ep` with a `color` pawn.
        let mut from_bb =
            PAWN_ATTACKS[color.flip().to_u8() as usize][ep.to_u8() as usize]
                & board.pieces_of(Piece::Pawn, color);
        while let Some(from) = from_bb.pop_lsb() {
            moves.push(
                Move::new(from, ep).with_flags(Move::CAPTURE | Move::EN_PASSANT),
            );
        }
    }
}

fn push_pawn_move(
    moves: &mut MoveList,
    from: Square,
    to: Square,
    promo_rank: Rank,
    flags: u32,
) {
    if to.rank() == promo_rank {
        moves.push(Move::new_promotion(from, to, Piece::Queen).with_flags(flags));
        moves.push(Move::new_promotion(from, to, Piece::Rook).with_flags(flags));
        moves.push(Move::new_promotion(from, to, Piece::Bishop).with_flags(flags));
        moves.push(Move::new_promotion(from, to, Piece::Knight).with_flags(flags));
    } else {
        moves.push(Move::new(from, to).with_flags(flags));
    }
}

fn generate_castling(board: &Board, color: Color, empty: Bitboard, moves: &mut MoveList) {
    let rights = board.castling();
    let enemy = color.flip();

    match color {
        Color::White => {
            if rights.contains(CastlingRights::WHITE_KING)
                && empty.contains(Square::F1)
                && empty.contains(Square::G1)
                && board.piece_at(Square::H1) == Some((Piece::Rook, Color::White))
                && !is_square_attacked(board, Square::E1, enemy)
                && !is_square_attacked(board, Square::F1, enemy)
                && !is_square_attacked(board, Square::G1, enemy)
            {
                moves.push(Move::new(Square::E1, Square::G1).with_flags(Move::CASTLE));
            }
            if rights.contains(CastlingRights::WHITE_QUEEN)
                && empty.contains(Square::D1)
                && empty.contains(Square::C1)
                && empty.contains(Square::B1)
                && board.piece_at(Square::A1) == Some((Piece::Rook, Color::White))
                && !is_square_attacked(board, Square::E1, enemy)
                && !is_square_attacked(board, Square::D1, enemy)
                && !is_square_attacked(board, Square::C1, enemy)
            {
                moves.push(Move::new(Square::E1, Square::C1).with_flags(Move::CASTLE));
            }
        }
        Color::Black => {
            if rights.contains(CastlingRights::BLACK_KING)
                && empty.contains(Square::F8)
                && empty.contains(Square::G8)
                && board.piece_at(Square::H8) == Some((Piece::Rook, Color::Black))
                && !is_square_attacked(board, Square::E8, enemy)
                && !is_square_attacked(board, Square::F8, enemy)
                && !is_square_attacked(board, Square::G8, enemy)
            {
                moves.push(Move::new(Square::E8, Square::G8).with_flags(Move::CASTLE));
            }
            if rights.contains(CastlingRights::BLACK_QUEEN)
                && empty.contains(Square::D8)
                && empty.contains(Square::C8)
                && empty.contains(Square::B8)
                && board.piece_at(Square::A8) == Some((Piece::Rook, Color::Black))
                && !is_square_attacked(board, Square::E8, enemy)
                && !is_square_attacked(board, Square::D8, enemy)
                && !is_square_attacked(board, Square::C8, enemy)
            {
                moves.push(Move::new(Square::E8, Square::C8).with_flags(Move::CASTLE));
            }
        }
    }
}

/// Count nodes at `depth` using legal move generation.
#[must_use]
pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = MoveList::new();
    generate_legal(board, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0u64;
    for &m in moves.as_slice() {
        let undo = board.make_move(m);
        nodes += perft(board, depth - 1);
        board.unmake_move(m, undo);
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::file::File;
    use crate::board::fen::STARTPOS_FEN;
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
            assert_eq!(
                bishop::attacks(sq, Bitboard::EMPTY),
                bishop::BISHOP_ATTACKS[i]
            );
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
        board.set_piece(Piece::King, Color::White, Square::E1);
        board.set_piece(Piece::King, Color::Black, Square::E8);
        board.set_piece(Piece::Knight, Color::White, Square::B1);
        board.set_piece(Piece::Knight, Color::White, Square::G1);
        board.set_piece(Piece::Pawn, Color::White, Square::A3);
        board.set_piece(Piece::Pawn, Color::White, Square::C3);
        board.set_piece(Piece::Pawn, Color::White, Square::F3);
        board.set_piece(Piece::Pawn, Color::White, Square::H3);
        board.recompute_hash();

        let mut moves = MoveList::new();
        generate_pseudo_legal(&board, &mut moves);

        assert!(!moves.is_empty());
        assert!(
            moves
                .as_slice()
                .contains(&Move::new(Square::B1, Square::D2))
        );
        assert!(
            moves
                .as_slice()
                .contains(&Move::new(Square::G1, Square::E2))
        );
    }

    #[test]
    fn startpos_not_in_check() {
        let board = Board::startpos();
        assert!(!in_check(&board, Color::White));
        assert!(!in_check(&board, Color::Black));
    }

    #[test]
    fn make_unmake_restores_hash() {
        let mut board = Board::startpos();
        let before = board.hash();
        let mut moves = MoveList::new();
        generate_legal(&mut board, &mut moves);
        assert!(!moves.is_empty());
        let m = moves.as_slice()[0];
        let undo = board.make_move(m);
        board.unmake_move(m, undo);
        assert_eq!(board.hash(), before);
        assert_eq!(board.to_fen(), STARTPOS_FEN);
    }

    #[test]
    fn perft_startpos_depth_1_to_3() {
        let mut board = Board::startpos();
        assert_eq!(perft(&mut board, 1), 20);
        assert_eq!(perft(&mut board, 2), 400);
        assert_eq!(perft(&mut board, 3), 8902);
    }

    #[test]
    fn perft_kiwipete_depth_2() {
        // "Kiwipete" — exercises castling, EP, promotions.
        const FEN: &str =
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let Ok(mut board) = Board::from_fen(FEN) else {
            return;
        };
        assert_eq!(perft(&mut board, 1), 48);
        assert_eq!(perft(&mut board, 2), 2039);
    }

    #[test]
    fn perft_position_3_depth_3() {
        // En passant heavy position.
        const FEN: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let Ok(mut board) = Board::from_fen(FEN) else {
            return;
        };
        assert_eq!(perft(&mut board, 1), 14);
        assert_eq!(perft(&mut board, 2), 191);
        assert_eq!(perft(&mut board, 3), 2812);
    }
}
