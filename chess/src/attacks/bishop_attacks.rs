use crate::{Bitboard, Square};

pub struct BishopAttacks;
impl BishopAttacks {
    #[inline]
    pub fn get_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        let mask = BISHOP[usize::from(square)];

        let mut diagonal = occupancy & mask.diagonal;
        let mut reverse_diagonal = diagonal.flip();
        diagonal = diagonal.wrapping_sub(Bitboard::from(mask.bitboard));
        reverse_diagonal = reverse_diagonal.wrapping_sub(Bitboard::from(mask.swap));
        diagonal ^= reverse_diagonal.flip();
        diagonal &= mask.diagonal;

        let mut anti = occupancy & mask.anti;
        let mut reverse_diagonal = anti.flip();
        anti = anti.wrapping_sub(Bitboard::from(mask.bitboard));
        reverse_diagonal = reverse_diagonal.wrapping_sub(Bitboard::from(mask.swap));
        anti ^= reverse_diagonal.flip();
        anti &= mask.anti;

        diagonal | anti
    }
}

#[derive(Clone, Copy, Default)]
struct Mask {
    bitboard: u64,
    swap: u64,
    diagonal: u64,
    anti: u64,
}

const BISHOP: [Mask; 64] = {
    let mut result = [Mask {
        bitboard: 0,
        swap: 0,
        diagonal: 0,
        anti: 0,
    }; 64];

    let mut square_index = 0;
    while square_index < 64 {
        let bit = 1u64 << square_index;
        let square = Square::from_value(square_index as u8);
        let file = square.file() as usize;
        let rank = square.get_rank() as usize;
        result[square_index] = Mask {
            bitboard: bit,
            swap: bit.swap_bytes(),
            diagonal: bit ^ DIAGS[7 + file - rank],
            anti: bit ^ DIAGS[file + rank].swap_bytes(),
        };
        square_index += 1;
    }

    result
};

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
