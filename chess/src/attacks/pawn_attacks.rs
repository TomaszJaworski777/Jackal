use crate::{base_structures::Side, Bitboard, Square};

pub struct PawnsAttacks;
impl PawnsAttacks {
    pub const ATTACK_TABLE: [[Bitboard; 64]; 2] = {
        let mut result = [[Bitboard::EMPTY; 64]; 2];
        let mut square_index = 0usize;
        while square_index < 64 {
            let bb = Bitboard::from_square(Square::from_value(square_index as u8));
            let mut attack_map: u64 = 0;
            if Bitboard::FILE_A
                .inverse()
                .and(bb.shift_left(9))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(9).get_value()
            }
            if Bitboard::FILE_H
                .inverse()
                .and(bb.shift_left(7))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(7).get_value()
            }
            result[Side::WHITE.get_value() as usize][square_index] =
                Bitboard::from_value(attack_map);
            attack_map = 0;
            if Bitboard::FILE_A
                .inverse()
                .and(bb.shift_right(7))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(7).get_value()
            }
            if Bitboard::FILE_H
                .inverse()
                .and(bb.shift_right(9))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(9).get_value()
            }
            result[Side::BLACK.get_value() as usize][square_index] =
                Bitboard::from_value(attack_map);
            square_index += 1;
        }
        result
    };
}
