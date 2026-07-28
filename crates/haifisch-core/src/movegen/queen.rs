use crate::board::{bitboard::Bitboard, square::Square};
use crate::movegen::{bishop, rook};

const fn generate_queen_attacks() -> [Bitboard; 64] {
    let mut attacks = [Bitboard::EMPTY; 64];

    let mut sq = 0;
    while sq < 64 {
        attacks[sq] = Bitboard::from_u64(
            bishop::BISHOP_ATTACKS[sq].to_u64() | rook::ROOK_ATTACKS[sq].to_u64(),
        );
        sq += 1;
    }

    attacks
}

pub static QUEEN_ATTACKS: [Bitboard; 64] = generate_queen_attacks();

/// Queen attacks from `square` given `occupancy` (blockers inclusive).
#[inline]
#[must_use]
pub fn attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    bishop::attacks(square, occupancy) | rook::attacks(square, occupancy)
}
