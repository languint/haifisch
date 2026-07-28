use crate::board::{bitboard::Bitboard, color::Color, file::File, rank::Rank, square::Square};

const fn generate_pawn_attacks() -> [[Bitboard; 64]; 2] {
    let mut attacks = [[Bitboard::EMPTY; 64]; 2];

    let mut sq = 0;
    while sq < 64 {
        #[allow(clippy::cast_possible_truncation)]
        let square = unsafe { Square::from_u8_unchecked(sq as u8) };

        let file = square.file().to_u8().cast_signed();
        let rank = square.rank().to_u8().cast_signed();

        // White captures: northeast / northwest
        let mut white = Bitboard::EMPTY;
        let mut i = 0;
        let white_deltas = [(-1_i8, 1_i8), (1_i8, 1_i8)];
        while i < 2 {
            let (df, dr) = white_deltas[i];
            let nf = file + df;
            let nr = rank + dr;
            if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 {
                white = Bitboard::from_u64(
                    white.to_u64()
                        | Bitboard::mask_from_parts(
                            unsafe { File::from_u8_unchecked(nf.cast_unsigned()) },
                            unsafe { Rank::from_u8_unchecked(nr.cast_unsigned()) },
                        )
                        .to_u64(),
                );
            }
            i += 1;
        }

        // Black captures: southeast / southwest
        let mut black = Bitboard::EMPTY;
        i = 0;
        let black_deltas = [(-1_i8, -1_i8), (1_i8, -1_i8)];
        while i < 2 {
            let (df, dr) = black_deltas[i];
            let nf = file + df;
            let nr = rank + dr;
            if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 {
                black = Bitboard::from_u64(
                    black.to_u64()
                        | Bitboard::mask_from_parts(
                            unsafe { File::from_u8_unchecked(nf.cast_unsigned()) },
                            unsafe { Rank::from_u8_unchecked(nr.cast_unsigned()) },
                        )
                        .to_u64(),
                );
            }
            i += 1;
        }

        attacks[Color::White.to_u8() as usize][sq] = white;
        attacks[Color::Black.to_u8() as usize][sq] = black;
        sq += 1;
    }

    attacks
}

pub static PAWN_ATTACKS: [[Bitboard; 64]; 2] = generate_pawn_attacks();
