use crate::board::{bitboard::Bitboard, file::File, rank::Rank, square::Square};

const BISHOP_DELTAS: [(i8, i8); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

const fn generate_bishop_attacks() -> [Bitboard; 64] {
    let mut attacks = [Bitboard::EMPTY; 64];

    let mut sq = 0;
    while sq < 64 {
        #[allow(clippy::cast_possible_truncation)]
        let square = unsafe { Square::from_u8_unchecked(sq as u8) };

        let file = square.file().to_u8().cast_signed();
        let rank = square.rank().to_u8().cast_signed();

        let mut bb = Bitboard::EMPTY;

        let mut i = 0;
        while i < 4 {
            let (df, dr) = BISHOP_DELTAS[i];

            let mut nf = file + df;
            let mut nr = rank + dr;

            while nf >= 0 && nf < 8 && nr >= 0 && nr < 8 {
                bb = Bitboard::from_u64(
                    bb.to_u64()
                        | Bitboard::mask_from_parts(
                            unsafe { File::from_u8_unchecked(nf.cast_unsigned()) },
                            unsafe { Rank::from_u8_unchecked(nr.cast_unsigned()) },
                        )
                        .to_u64(),
                );

                nf += df;
                nr += dr;
            }

            i += 1;
        }

        attacks[sq] = bb;
        sq += 1;
    }

    attacks
}

pub static BISHOP_ATTACKS: [Bitboard; 64] = generate_bishop_attacks();

/// Bishop attacks from `square` given `occupancy` (blockers inclusive).
#[inline]
#[must_use]
pub fn attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    let file = square.file().to_u8().cast_signed();
    let rank = square.rank().to_u8().cast_signed();

    let mut bb = Bitboard::EMPTY;

    for &(df, dr) in &BISHOP_DELTAS {
        let mut nf = file + df;
        let mut nr = rank + dr;

        while (0..8).contains(&nf) && (0..8).contains(&nr) {
            let to = Square::from_parts(
                unsafe { File::from_u8_unchecked(nf.cast_unsigned()) },
                unsafe { Rank::from_u8_unchecked(nr.cast_unsigned()) },
            );
            bb |= Bitboard::mask_from_square(to);
            if occupancy.contains(to) {
                break;
            }
            nf += df;
            nr += dr;
        }
    }

    bb
}
