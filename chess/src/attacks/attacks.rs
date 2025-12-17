use crate::{
    Bitboard, Square, attacks::{KingAttacks, KnightAttacks, PawnsAttacks, slider_attacks}, base_structures::Side
};

pub struct Attacks;
impl Attacks {
    #[inline(always)]
    pub const fn get_king_attacks(square: Square) -> Bitboard {
        KingAttacks::ATTACK_TABLE[square.get_value() as usize]
    }

    #[inline(always)]
    pub const fn get_knight_attacks(square: Square) -> Bitboard {
        KnightAttacks::ATTACK_TABLE[square.get_value() as usize]
    }

    #[inline(always)]
    pub const fn get_pawn_attacks(square: Square, attacker_side: Side) -> Bitboard {
        PawnsAttacks::ATTACK_TABLE[attacker_side.get_value() as usize][square.get_value() as usize]
    }

    #[inline(always)]
    pub fn get_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        unsafe {
            let params = &slider_attacks::LOOKUP[64 + square.get_value() as usize];
            let idx = slider_attacks::calculate_index(occupancy.get_value(), params);
            Bitboard::from_value(*slider_attacks::ATTACK_TABLE.get_unchecked(params.offset as usize + idx))
        }
    }

    #[inline(always)]
    pub fn get_rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        unsafe {
            let params = &slider_attacks::LOOKUP[square.get_value() as usize];
            let idx = slider_attacks::calculate_index(occupancy.get_value(), params);
            Bitboard::from_value(*slider_attacks::ATTACK_TABLE.get_unchecked(params.offset as usize + idx))
        } 
    }
}
