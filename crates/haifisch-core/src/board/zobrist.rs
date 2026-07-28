use crate::board::{castling::CastlingRights, color::Color, file::File, piece::Piece, square::Square};

/// Zobrist keys for incremental hashing.
pub struct Zobrist;

impl Zobrist {
    /// `PIECE[color][piece][square]`
    pub const PIECE: [[[u64; 64]; 6]; 2] = generate_piece_keys();
    pub const SIDE: u64 = generate_side_key();
    /// Indexed by [`CastlingRights::bits`].
    pub const CASTLING: [u64; 16] = generate_castling_keys();
    /// Indexed by file 0–7; unused when there is no EP square.
    pub const EP_FILE: [u64; 8] = generate_ep_keys();

    #[inline]
    #[must_use]
    pub const fn piece(color: Color, piece: Piece, square: Square) -> u64 {
        Self::PIECE[color.to_u8() as usize][piece.to_u8() as usize][square.to_u8() as usize]
    }

    #[inline]
    #[must_use]
    pub const fn castling(rights: CastlingRights) -> u64 {
        Self::CASTLING[rights.bits() as usize]
    }

    #[inline]
    #[must_use]
    pub const fn ep_file(file: File) -> u64 {
        Self::EP_FILE[file.to_u8() as usize]
    }
}

const SEED: u64 = 0xA1B2_C3D4_E5F6_7788;

const fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

const fn generate_piece_keys() -> [[[u64; 64]; 6]; 2] {
    let mut keys = [[[0u64; 64]; 6]; 2];
    let mut state = SEED;
    let mut color = 0;
    while color < 2 {
        let mut piece = 0;
        while piece < 6 {
            let mut sq = 0;
            while sq < 64 {
                keys[color][piece][sq] = splitmix64(&mut state);
                sq += 1;
            }
            piece += 1;
        }
        color += 1;
    }
    keys
}

const fn generate_side_key() -> u64 {
    let mut state = SEED ^ 0xDEAD_BEEF_CAFE_BABE;
    // Advance past piece keys for a distinct stream.
    let mut i = 0;
    while i < 2 * 6 * 64 {
        let _ = splitmix64(&mut state);
        i += 1;
    }
    splitmix64(&mut state)
}

const fn generate_castling_keys() -> [u64; 16] {
    let mut keys = [0u64; 16];
    let mut state = SEED ^ 0x0123_4567_89AB_CDEF;
    let mut i = 0;
    while i < 2 * 6 * 64 + 1 {
        let _ = splitmix64(&mut state);
        i += 1;
    }
    i = 0;
    while i < 16 {
        keys[i] = splitmix64(&mut state);
        i += 1;
    }
    keys
}

const fn generate_ep_keys() -> [u64; 8] {
    let mut keys = [0u64; 8];
    let mut state = SEED ^ 0xF0E1_D2C3_B4A5_9687;
    let mut i = 0;
    while i < 2 * 6 * 64 + 1 + 16 {
        let _ = splitmix64(&mut state);
        i += 1;
    }
    i = 0;
    while i < 8 {
        keys[i] = splitmix64(&mut state);
        i += 1;
    }
    keys
}
