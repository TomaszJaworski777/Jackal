use crate::spear::Bitboard;

pub struct KingAttacks;
impl KingAttacks {
    pub const ATTACK_TABLE: [Bitboard; 64] = {
        let mut result = [Bitboard::EMPTY; 64];
        let mut square_index = 0usize;
        while square_index < 64 {
            let bb = Bitboard::from_raw(1u64 << square_index);
            let mut attack_map: u64 = 0;
            if Bitboard::FILE_H
                .inverse()
                .and(bb.shift_left(7))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(7).get_raw()
            }
            attack_map |= bb.shift_left(8).get_raw();
            if Bitboard::FILE_A
                .inverse()
                .and(bb.shift_left(9))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(9).get_raw()
            }
            if Bitboard::FILE_A
                .inverse()
                .and(bb.shift_right(7))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(7).get_raw()
            }
            attack_map |= bb.shift_right(8).get_raw();
            if Bitboard::FILE_H
                .inverse()
                .and(bb.shift_right(9))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(9).get_raw()
            }
            if Bitboard::FILE_A
                .inverse()
                .and(bb.shift_left(1))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(1).get_raw()
            }
            if Bitboard::FILE_H
                .inverse()
                .and(bb.shift_right(1))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(1).get_raw()
            }
            result[square_index] = Bitboard::from_raw(attack_map);
            square_index += 1;
        }
        result
    };
}
