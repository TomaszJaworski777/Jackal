use crate::spear::{Bitboard, Square};

pub struct RookAttacks;
impl RookAttacks {
    #[inline]
    pub fn get_rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        let flip = ((occupancy >> (square.get_raw() as u32 & 7)) & Bitboard::FILE_A).wrapping_mul(Bitboard::from_raw(DIAG));
        let file_sq = (flip >> 57) & 0x3F;
        let files = FILE[square.get_raw() as usize][file_sq.get_raw() as usize];

        let rank_sq = (occupancy >> RANK_SHIFT[square.get_raw() as usize] as u32) & 0x3F;
        let ranks = RANK[square.get_raw() as usize][rank_sq.get_raw() as usize];

        Bitboard::from_raw(ranks | files)
    }
}

const DIAG: u64 = 0x8040_2010_0804_0201;

const EAST: [u64; 64] = {
    let mut result = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        result[sq] = (0xFF << (sq & 56)) ^ (1 << sq) ^ WEST[sq];
        sq += 1;
    }

    result
};

const WEST: [u64; 64] = {
    let mut result = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        result[sq] = (0xFF << (sq & 56)) & ((1 << sq) - 1);
        sq += 1;
    }

    result
};

const RANK_SHIFT: [usize; 64] = {
    let mut result = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        result[sq] = sq - (sq & 7) + 1;
        sq += 1;
    }

    result
};

static RANK: [[u64; 64]; 64] =  {
    let mut result = [[0; 64]; 64];
    let mut sq = 0;
    while sq < 64 {
        let mut occ = 0;
        while occ < 64 {
            let file = sq & 7;
            let mask = (occ << 1) as u64;
            let east = ((EAST[file] & mask) | (1 << 63)).trailing_zeros() as usize;
            let west = ((WEST[file] & mask) | 1).leading_zeros() as usize ^ 63;
            result[sq][occ] = (EAST[file] ^ EAST[east] | WEST[file] ^ WEST[west]) << (sq - file);
            occ += 1;
        }
        sq += 1;
    }

    result
};


static FILE: [[u64; 64]; 64] = {
    let mut result = [[0; 64]; 64];
    let mut sq = 0;
    while sq < 64 {
        let mut occ = 0;
        while occ < 64 {
            result[sq][occ] = (Bitboard::FILE_H.and(Bitboard::from_raw(RANK[7 - sq / 8][occ].wrapping_mul(DIAG)))).shift_right(7 - (sq & 7) as u32).get_raw();
            occ += 1;
        }
        sq += 1;
    }

    result
};