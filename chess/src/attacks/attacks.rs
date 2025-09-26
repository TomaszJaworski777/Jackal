use crate::{
    attacks::{BishopAttacks, KingAttacks, KnightAttacks, PawnsAttacks, RookAttacks},
    base_structures::Side,
    Bitboard, Square,
};

pub struct Attacks;
impl Attacks {
    #[inline]
    pub fn get_king_attacks(square: Square) -> Bitboard {
        KingAttacks::ATTACK_TABLE[usize::from(square)]
    }

    #[inline]
    pub fn get_knight_attacks(square: Square) -> Bitboard {
        KnightAttacks::ATTACK_TABLE[usize::from(square)]
    }

    #[inline]
    pub fn get_pawn_attacks(square: Square, attacker_side: Side) -> Bitboard {
        PawnsAttacks::ATTACK_TABLE[usize::from(attacker_side)][usize::from(square)]
    }

    #[inline]
    pub fn get_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        BishopAttacks::get_bishop_attacks(square, occupancy)
    }

    #[inline]
    pub fn get_rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        RookAttacks::get_rook_attacks(square, occupancy)
    }
}
