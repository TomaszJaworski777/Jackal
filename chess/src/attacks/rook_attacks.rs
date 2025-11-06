use crate::{Bitboard, Square};

pub struct RookAttacks;
impl RookAttacks {
    #[inline]
    pub fn get_rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        let flip = ((occupancy >> (u8::from(square) & 7)) & Bitboard::FILE_A)
            .wrapping_mul(Bitboard::from(0x8040_2010_0804_0201));
        let occ = (flip >> 57) & 0x3F;
        let file = FILE[usize::from(square)][u64::from(occ) as usize];

        let occ = (occupancy >> RANK_SHIFT[usize::from(square)] as u8) & 0x3F;
        let rank = RANK[usize::from(square)][u64::from(occ) as usize];

        Bitboard::from(rank | file)
    }
}

const EAST: [u64; 64] = {
    let mut result = [0; 64];
    let mut square_index = 0;
    while square_index < 64 {
        result[square_index] =
            (0xFF << (square_index & 56)) ^ (1 << square_index) ^ WEST[square_index];
        square_index += 1;
    }

    result
};

const WEST: [u64; 64] = {
    let mut result = [0; 64];
    let mut square_index = 0;
    while square_index < 64 {
        result[square_index] = (0xFF << (square_index & 56)) & ((1 << square_index) - 1);
        square_index += 1;
    }

    result
};

const RANK_SHIFT: [usize; 64] = {
    let mut result = [0; 64];
    let mut square_index = 0;
    while square_index < 64 {
        result[square_index] = square_index - (square_index & 7) + 1;
        square_index += 1;
    }

    result
};

const RANK: [[u64; 64]; 64] = {
    let mut result = [[0; 64]; 64];
    let mut square_index = 0;
    while square_index < 64 {
        let mut occupancy = 0;
        while occupancy < 64 {
            let file = square_index & 7;
            let mask = (occupancy << 1) as u64;
            let east = ((EAST[file] & mask) | (1 << 63)).trailing_zeros() as usize;
            let west = ((WEST[file] & mask) | 1).leading_zeros() as usize ^ 63;
            result[square_index][occupancy] =
                (EAST[file] ^ EAST[east] | WEST[file] ^ WEST[west]) << (square_index - file);
            occupancy += 1;
        }
        square_index += 1;
    }

    result
};

const FILE: [[u64; 64]; 64] = {
    let mut result = [[0; 64]; 64];
    let mut square_index = 0;
    while square_index < 64 {
        let mut occupancy = 0;
        let square = Square::from_value(square_index as u8);
        while occupancy < 64 {
            result[square_index][occupancy] = (Bitboard::FILE_H.and(Bitboard::from_value(
                RANK[7 - square.get_rank() as usize][occupancy].wrapping_mul(0x8040_2010_0804_0201),
            )))
            .shift_right(7 - square.file() as u32)
            .get_value();
            occupancy += 1;
        }
        square_index += 1;
    }

    result
};
