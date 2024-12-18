use crate::spear::{
    attacks::{Attacks, Rays},
    Bitboard, ChessBoard, Piece,
};

#[derive(Clone, Copy, Default)]
pub struct ChessBoardMasks {}

impl ChessBoard {
    #[inline]
    pub fn is_in_check<const DEFENDER_WHITE: bool, const ATTACKER_WHITE: bool>(&self) -> bool {
        self.is_square_attacked::<DEFENDER_WHITE, ATTACKER_WHITE>(
            self.get_king_square::<DEFENDER_WHITE>(),
        )
    }

    #[inline]
    pub fn generate_checkers_mask<const DEFENDER_WHITE: bool, const ATTACKER_WHITE: bool>(
        &self,
    ) -> Bitboard {
        self.all_attackers_to_square::<DEFENDER_WHITE, ATTACKER_WHITE>(
            self.get_occupancy(),
            self.get_king_square::<DEFENDER_WHITE>(),
        )
    }

    pub fn generate_pin_masks<const DEFENDER_WHITE: bool, const ATTACKER_WHITE: bool>(
        &self,
    ) -> (Bitboard, Bitboard) {
        let king_square = self.get_king_square::<DEFENDER_WHITE>();
        let defender_occupancy = self.get_occupancy_for_side::<DEFENDER_WHITE>();
        let attacker_occupancy = self.get_occupancy_for_side::<ATTACKER_WHITE>();
        let queens = self.get_piece_mask_for_side::<ATTACKER_WHITE>(Piece::QUEEN);

        let potential_pinners =
            Attacks::get_bishop_attacks_for_square(king_square, attacker_occupancy)
                & (self.get_piece_mask_for_side::<ATTACKER_WHITE>(Piece::BISHOP) | queens);

        let mut diag_result = Bitboard::EMPTY;
        potential_pinners.map(|potential_pinner| {
            let ray = Rays::get_ray(king_square, potential_pinner);
            if (ray & defender_occupancy).only_one_bit() {
                diag_result |= ray;
            }
        });

        let potential_pinners =
            Attacks::get_rook_attacks_for_square(king_square, attacker_occupancy)
                & (self.get_piece_mask_for_side::<ATTACKER_WHITE>(Piece::ROOK) | queens);
        let mut orto_result = Bitboard::EMPTY;
        potential_pinners.map(|potential_pinner| {
            let ray = Rays::get_ray(king_square, potential_pinner);
            if (ray & defender_occupancy).only_one_bit() {
                orto_result |= ray;
            }
        });

        (diag_result, orto_result)
    }

    pub fn generate_attack_map<const DEFENDER_WHITE: bool, const ATTACKER_WHITE: bool>(
        &self,
    ) -> Bitboard {
        let mut threats = Bitboard::EMPTY;

        let king_square = self.get_king_square::<DEFENDER_WHITE>();
        let occupancy = self.get_occupancy() ^ king_square.get_bit();

        let attacker_pieces = self.get_occupancy_for_side::<ATTACKER_WHITE>();
        let queens = self.get_piece_mask(Piece::QUEEN);

        (attacker_pieces & (self.get_piece_mask(Piece::ROOK) | queens)).map(|rook_square| {
            threats |= Attacks::get_rook_attacks_for_square(rook_square, occupancy)
        });

        (attacker_pieces & (self.get_piece_mask(Piece::BISHOP) | queens)).map(|bishop_square| {
            threats |= Attacks::get_bishop_attacks_for_square(bishop_square, occupancy)
        });

        (attacker_pieces & self.get_piece_mask(Piece::KING))
            .map(|king_square| threats |= Attacks::get_king_attacks_for_square(king_square));

        (attacker_pieces & self.get_piece_mask(Piece::KNIGHT))
            .map(|knight_square| threats |= Attacks::get_knight_attacks_for_square(knight_square));

        (attacker_pieces & self.get_piece_mask(Piece::PAWN)).map(|pawn_square| {
            threats |= Attacks::get_pawn_attacks_for_square::<ATTACKER_WHITE>(pawn_square)
        });

        threats
    }
}
