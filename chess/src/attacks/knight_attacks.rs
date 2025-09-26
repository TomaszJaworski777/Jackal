use crate::{Bitboard, Square};

pub struct KnightAttacks;
impl KnightAttacks {
    pub const ATTACK_TABLE: [Bitboard; 64] = {
        let mut result = [Bitboard::EMPTY; 64];
        let mut square_index = 0usize;
        while square_index < 64 {
            let bb = Bitboard::from_square(Square::from_value(square_index as u8));
            let mut attack_map: u64 = 0;
            if Bitboard::FILE_A
                .inverse()
                .and(bb.shift_left(17))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(17).get_value()
            }
            if Bitboard::FILE_H
                .inverse()
                .and(bb.shift_left(15))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(15).get_value()
            }
            if Bitboard::FILE_A
                .or(Bitboard::FILE_B)
                .inverse()
                .and(bb.shift_left(10))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(10).get_value()
            }
            if Bitboard::FILE_H
                .or(Bitboard::FILE_G)
                .inverse()
                .and(bb.shift_left(6))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(6).get_value()
            }
            if Bitboard::FILE_H
                .inverse()
                .and(bb.shift_right(17))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(17).get_value()
            }
            if Bitboard::FILE_A
                .inverse()
                .and(bb.shift_right(15))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(15).get_value()
            }
            if Bitboard::FILE_H
                .or(Bitboard::FILE_G)
                .inverse()
                .and(bb.shift_right(10))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(10).get_value()
            }
            if Bitboard::FILE_A
                .or(Bitboard::FILE_B)
                .inverse()
                .and(bb.shift_right(6))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(6).get_value()
            }
            result[square_index] = Bitboard::from_value(attack_map);
            square_index += 1;
        }
        result
    };
}
