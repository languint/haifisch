use crate::board::{bitboard::Bitboard, file::File, rank::Rank, square::Square};

const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (-2, -1),
    (-2, 1),
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
    (2, -1),
    (2, 1),
];

const fn generate_knight_attacks() -> [Bitboard; 64] {
    let mut attacks = [Bitboard::EMPTY; 64];

    let mut sq = 0;
    while sq < 64 {
        #[allow(clippy::cast_possible_truncation)]
        let square = unsafe { Square::from_u8_unchecked(sq as u8) };

        let file = square.file().to_u8().cast_signed();
        let rank = square.rank().to_u8().cast_signed();

        let mut bb = Bitboard::EMPTY;

        let mut i = 0;
        while i < 8 {
            let (df, dr) = KNIGHT_DELTAS[i];

            let nf = file + df;
            let nr = rank + dr;

            if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 {
                bb = Bitboard::from_u64(
                    bb.to_u64()
                        | Bitboard::mask_from_parts(
                            unsafe { File::from_u8_unchecked(nf.cast_unsigned()) },
                            unsafe { Rank::from_u8_unchecked(nr.cast_unsigned()) },
                        )
                        .to_u64(),
                );
            }

            i += 1;
        }

        attacks[sq] = bb;
        sq += 1;
    }

    attacks
}

pub static KNIGHT_ATTACKS: [Bitboard; 64] = generate_knight_attacks();
