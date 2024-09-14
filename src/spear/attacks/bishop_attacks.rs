#[cfg(target_feature = "bmi2")]
use std::arch::x86_64::_pext_u64;

use once_cell::sync::Lazy;

use crate::spear::{Bitboard, Square};

pub struct BishopAttacks;
impl BishopAttacks {
    #[inline]
    pub fn get_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        let square = usize::from(square);

        #[cfg(not(target_feature = "bmi2"))]
        let index = ((occupancy & BISHOP_MASKS[square])
            .wrapping_mul(MAGIC_NUMBERS_BISHOP[square].into())
            >> (64 - BISHOP_OCCUPANCY_COUNT[square] as u32))
            .get_raw() as usize;

        #[cfg(target_feature = "bmi2")]
        let index =
            unsafe { _pext_u64(occupancy.get_raw(), BISHOP_MASKS[square].get_raw()) as usize };

        BISHOP_ATTACKS[square][index]
    }
}

const BISHOP_MASKS: [Bitboard; 64] = {
    let mut result = [Bitboard::EMPTY; 64];
    let mut square_index = 0u8;
    while square_index < 64 {
        result[square_index as usize] = mask_bishop_attacks(Square::from_raw(square_index));
        square_index += 1;
    }
    result
};

#[cfg(not(target_feature = "bmi2"))]
const BISHOP_OCCUPANCY_COUNT: [usize; 64] = {
    let mut result = [0; 64];
    let mut rank = 0;
    while rank < 8 {
        let mut file = 0;
        while file < 8 {
            let square = Square::from_coords(rank, file);
            result[square.get_raw() as usize] = mask_bishop_attacks(square).pop_count() as usize;
            file += 1;
        }
        rank += 1;
    }
    result
};

static BISHOP_ATTACKS: Lazy<Vec<Vec<Bitboard>>> = Lazy::new(|| {
    let mut result = vec![vec![Bitboard::EMPTY; 512]; 64];
    for square_index in 0..64 {
        let square = Square::from_raw(square_index);
        let attack_mask = mask_bishop_attacks(square);
        let relevant_bit_count = attack_mask.pop_count();
        let mut index = 0;
        while index < 1 << relevant_bit_count {
            let occupancy = generate_occupancy(index, relevant_bit_count as usize, attack_mask);

            #[cfg(not(target_feature = "bmi2"))]
            let attack_index = (occupancy
                .wrapping_mul(MAGIC_NUMBERS_BISHOP[square_index as usize].into())
                >> (64 - relevant_bit_count))
                .get_raw() as usize;

            #[cfg(target_feature = "bmi2")]
            let attack_index = unsafe {
                _pext_u64(
                    occupancy.get_raw(),
                    BISHOP_MASKS[square_index as usize].get_raw(),
                ) as usize
            };

            result[square_index as usize][attack_index] =
                generate_bishop_attacks(square, occupancy);
            index += 1;
        }
    }

    result
});

const fn mask_bishop_attacks(square: Square) -> Bitboard {
    let mut result: u64 = 0;
    let bishop_position = (square.get_rank() as i32, square.get_file() as i32);

    let mut rank = bishop_position.0 + 1;
    let mut file = bishop_position.1 + 1;
    while rank < 7 && file < 7 {
        result |= Square::from_coords(rank as u8, file as u8)
            .get_bit()
            .get_raw();
        rank += 1;
        file += 1;
    }

    rank = bishop_position.0 - 1;
    file = bishop_position.1 + 1;
    while rank > 0 && file < 7 {
        result |= Square::from_coords(rank as u8, file as u8)
            .get_bit()
            .get_raw();
        rank -= 1;
        file += 1;
    }

    rank = bishop_position.0 - 1;
    file = bishop_position.1 - 1;
    while rank > 0 && file > 0 {
        result |= Square::from_coords(rank as u8, file as u8)
            .get_bit()
            .get_raw();
        rank -= 1;
        file -= 1;
    }

    rank = bishop_position.0 + 1;
    file = bishop_position.1 - 1;
    while rank < 7 && file > 0 {
        result |= Square::from_coords(rank as u8, file as u8)
            .get_bit()
            .get_raw();
        rank += 1;
        file -= 1;
    }

    Bitboard::from_raw(result)
}

fn generate_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    let mut result: Bitboard = Bitboard::EMPTY;
    let bishop_position = (square.get_rank() as i32, square.get_file() as i32);

    let mut rank = bishop_position.0 + 1;
    let mut file = bishop_position.1 + 1;
    while rank < 8 && file < 8 {
        result.set_bit(Square::from_coords(rank as u8, file as u8));
        if (Square::from_coords(rank as u8, file as u8).get_bit() & occupancy).is_not_empty() {
            break;
        }
        rank += 1;
        file += 1;
    }

    rank = bishop_position.0 - 1;
    file = bishop_position.1 + 1;
    while rank >= 0 && file < 8 {
        result.set_bit(Square::from_coords(rank as u8, file as u8));
        if (Square::from_coords(rank as u8, file as u8).get_bit() & occupancy).is_not_empty() {
            break;
        }
        rank -= 1;
        file += 1;
    }

    rank = bishop_position.0 - 1;
    file = bishop_position.1 - 1;
    while rank >= 0 && file >= 0 {
        result.set_bit(Square::from_coords(rank as u8, file as u8));
        if (Square::from_coords(rank as u8, file as u8).get_bit() & occupancy).is_not_empty() {
            break;
        }
        rank -= 1;
        file -= 1;
    }

    rank = bishop_position.0 + 1;
    file = bishop_position.1 - 1;
    while rank < 8 && file >= 0 {
        result.set_bit(Square::from_coords(rank as u8, file as u8));
        if (Square::from_coords(rank as u8, file as u8).get_bit() & occupancy).is_not_empty() {
            break;
        }
        rank += 1;
        file -= 1;
    }

    result
}

fn generate_occupancy(index: usize, bit_count: usize, attack_mask: Bitboard) -> Bitboard {
    let mut result = Bitboard::EMPTY;
    let mut mut_attack_mask = attack_mask;
    let mut count_index = 0u16;
    while count_index < bit_count as u16 {
        let square: Square = mut_attack_mask.pop_ls1b_square();
        if index & (1usize << count_index) > 0 {
            result.set_bit(square);
        }

        count_index += 1;
    }

    result
}

#[cfg(not(target_feature = "bmi2"))]
const MAGIC_NUMBERS_BISHOP: [u64; 64] = [
    9300092178686681120,
    1284830893973760,
    2322997520105472,
    16142031364873674789,
    10383348832699154706,
    571763293421568,
    37726495118197760,
    1513231473652670722,
    40550006146990185,
    873700543932137730,
    36037870288505856,
    431188982368272,
    1155210765395821056,
    11538293718411908608,
    4611721787053966849,
    103589390848170272,
    1125968899098624,
    144680358661721088,
    11259553153024529,
    10133272101128193,
    73751202732572676,
    144679238632472832,
    2357915965813425297,
    401383670122021888,
    13528392142225729,
    4643215615211930112,
    9226802530447557664,
    1302666467161997954,
    1306326466426847232,
    2253998841200772,
    9223935538715955216,
    4611977389012961280,
    1161101459345408,
    5630633405878272,
    154573777173479968,
    5224739618297217088,
    184790590020518016,
    141291540840712,
    4621296042111943168,
    9278545841721754664,
    13814550243590400,
    757176487873905668,
    2598717998437009408,
    2344123889522575360,
    360290349769303040,
    14053484853547533328,
    9227878118977438752,
    5102361295591936,
    5233754530306591776,
    4689658989384957952,
    1161642645719051,
    2252351784355840,
    2337004390424,
    1190112502864707589,
    290499785468691593,
    2387190454312566784,
    1235149585505599557,
    4683745820179825441,
    18014407116507136,
    1741698094928005,
    144749056665649409,
    576461028523640968,
    74921813755137,
    18085875364200714,
];
