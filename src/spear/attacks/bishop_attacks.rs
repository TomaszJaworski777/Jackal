use crate::spear::{Bitboard, Square};

pub struct BishopAttacks;
impl BishopAttacks {
    #[inline]
    pub fn get_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        let mask = BISHOP[square.get_raw() as usize];

        let mut diag = occupancy & mask.diag;
        let mut rev1 = diag.flip();
        diag = diag.wrapping_sub(Bitboard::from_raw(mask.bit));
        rev1 = rev1.wrapping_sub(Bitboard::from_raw(mask.swap));
        diag ^= rev1.flip();
        diag &= mask.diag;

        let mut anti = occupancy & mask.anti;
        let mut rev2 = anti.flip();
        anti = anti.wrapping_sub(Bitboard::from_raw(mask.bit));
        rev2 = rev2.wrapping_sub(Bitboard::from_raw(mask.swap));
        anti ^= rev2.flip();
        anti &= mask.anti;

        diag | anti
    }
}

#[derive(Clone, Copy, Default)]
struct Mask {
    bit: u64,
    diag: u64,
    anti: u64,
    swap: u64,
}

pub const DIAGS: [u64; 15] = [
    0x0100_0000_0000_0000,
    0x0201_0000_0000_0000,
    0x0402_0100_0000_0000,
    0x0804_0201_0000_0000,
    0x1008_0402_0100_0000,
    0x2010_0804_0201_0000,
    0x4020_1008_0402_0100,
    0x8040_2010_0804_0201,
    0x0080_4020_1008_0402,
    0x0000_8040_2010_0804,
    0x0000_0080_4020_1008,
    0x0000_0000_8040_2010,
    0x0000_0000_0080_4020,
    0x0000_0000_0000_8040,
    0x0000_0000_0000_0080,
];

const BISHOP: [Mask; 64] = {
    let mut result = [Mask { bit: 0, diag: 0, anti: 0, swap: 0 }; 64];

    let mut sq = 0;
    while sq < 64 {
        let bit = 1 << sq;
        let file = sq & 7;
        let rank = sq / 8;
        result[sq] = Mask {
            bit,
            diag: bit ^ DIAGS[7 + file - rank],
            anti: bit ^ DIAGS[    file + rank].swap_bytes(),
            swap: bit.swap_bytes()
        };
        sq += 1;
    }

    result
};