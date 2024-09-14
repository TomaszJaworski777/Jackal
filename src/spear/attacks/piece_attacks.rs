use crate::spear::{Bitboard, ChessBoard, Piece, Square};

use super::{
    bishop_attacks::BishopAttacks, king_attacks::KingAttacks, knight_attacks::KnightAttacks,
    pawn_attacks::PawnsAttacks, rook_attacks::RookAttacks,
};

pub struct Attacks;
impl Attacks {
    #[inline]
    pub fn get_king_attacks_for_square(square: Square) -> Bitboard {
        KingAttacks::ATTACK_TABLE[square.get_raw() as usize]
    }

    #[inline]
    pub fn get_knight_attacks_for_square(square: Square) -> Bitboard {
        KnightAttacks::ATTACK_TABLE[square.get_raw() as usize]
    }

    #[inline]
    pub fn get_pawn_attacks_for_square<const WHITE: bool>(square: Square) -> Bitboard {
        PawnsAttacks::ATTACK_TABLE[1 - usize::from(WHITE)][square.get_raw() as usize]
    }

    #[inline]
    pub fn get_bishop_attacks_for_square(square: Square, occupancy: Bitboard) -> Bitboard {
        BishopAttacks::get_bishop_attacks(square, occupancy)
    }

    #[inline]
    pub fn get_rook_attacks_for_square(square: Square, occupancy: Bitboard) -> Bitboard {
        RookAttacks::get_rook_attacks(square, occupancy)
    }
}

impl ChessBoard {
    pub fn all_attackers_to_square<const DEFENDER_WHITE: bool, const ATTACKER_WHITE: bool>(
        &self,
        occupancy: Bitboard,
        square: Square,
    ) -> Bitboard {
        let queens = self.get_piece_mask(Piece::QUEEN);
        ((Attacks::get_knight_attacks_for_square(square) & self.get_piece_mask(Piece::KNIGHT))
            | (Attacks::get_king_attacks_for_square(square) & self.get_piece_mask(Piece::KING))
            | (Attacks::get_pawn_attacks_for_square::<DEFENDER_WHITE>(square)
                & self.get_piece_mask(Piece::PAWN))
            | (Attacks::get_rook_attacks_for_square(square, occupancy)
                & (self.get_piece_mask(Piece::ROOK) | queens))
            | (Attacks::get_bishop_attacks_for_square(square, occupancy)
                & (self.get_piece_mask(Piece::BISHOP) | queens)))
            & self.get_occupancy_for_side::<ATTACKER_WHITE>()
    }

    pub fn is_square_attacked_with_occupancy<
        const DEFENDER_WHITE: bool,
        const ATTACKER_WHITE: bool,
    >(
        &self,
        square: Square,
        occupancy: Bitboard,
    ) -> bool {
        self.all_attackers_to_square::<DEFENDER_WHITE, ATTACKER_WHITE>(occupancy, square)
            .is_not_empty()
    }

    #[inline]
    pub fn is_square_attacked<const DEFENDER_WHITE: bool, const ATTACKER_WHITE: bool>(
        &self,
        square: Square,
    ) -> bool {
        self.is_square_attacked_with_occupancy::<DEFENDER_WHITE, ATTACKER_WHITE>(
            square,
            self.get_occupancy(),
        )
    }

    #[inline]
    pub fn is_square_attacked_with_attack_map(&self, square: Square, attack_map: Bitboard) -> bool {
        attack_map.get_bit(square)
    }
}
