use crate::spear::{Bitboard, Square};

pub struct Rays;
impl Rays {
    #[inline]
    pub fn get_ray(from: Square, to: Square) -> Bitboard {
        RAYS[from.get_raw() as usize][to.get_raw() as usize]
    }
}

const RAYS: [[Bitboard; 64]; 64] = {
    let mut result = [[Bitboard::EMPTY; 64]; 64];
    let mut from_square_index = 0;
    while from_square_index < 64 {
        let mut to_square_index = 0;
        while to_square_index < 64 {
            let from_square = Square::from_raw(from_square_index);
            let to_square = Square::from_raw(to_square_index);
            result[from_square_index as usize][to_square_index as usize] =
                generate_ray(from_square, to_square);
            to_square_index += 1;
        }
        from_square_index += 1;
    }

    result
};

const fn generate_ray(from: Square, to: Square) -> Bitboard {
    let rank_increment = (to.get_rank() as i32 - from.get_rank() as i32).signum();
    let file_increment = (to.get_file() as i32 - from.get_file() as i32).signum();

    if rank_increment == 0 && file_increment == 0 {
        return Bitboard::EMPTY;
    }

    let mut result = 0u64;
    let mut rank = from.get_rank() as i32 + rank_increment;
    let mut file = from.get_file() as i32 + file_increment;

    while rank >= 0 && rank <= 7 && file >= 0 && file <= 7 {
        let current_square = Square::from_coords(rank as u8, file as u8);
        result |= current_square.get_bit().get_raw();
        if to.equals(current_square) {
            return Bitboard::from_raw(result);
        }
        rank += rank_increment;
        file += file_increment;
    }

    Bitboard::EMPTY
}
