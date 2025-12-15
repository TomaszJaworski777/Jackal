use crate::{
    attacks::Rays,
    move_gen::piece_moves::{BISHOP, KNIGHT, ROOK},
    Bitboard, ChessBoard, Move, Side,
};

pub(super) const WHITE: u8 = 0;
pub(super) const BLACK: u8 = 1;

pub(super) struct MoveGen;

impl ChessBoard {
    #[inline]
    pub(crate) fn map_legal_moves_templated<F: FnMut(Move), const COLOR: u8>(
        &self,
        mut apply_move: F,
    ) {
        self.map_legal_moves_internal::<_, COLOR, false>(&mut apply_move)
    }

    #[inline]
    pub fn map_legal_moves<F: FnMut(Move)>(&self, mut apply_move: F) {
        if self.side() == Side::WHITE {
            self.map_legal_moves_internal::<_, WHITE, false>(&mut apply_move)
        } else {
            self.map_legal_moves_internal::<_, BLACK, false>(&mut apply_move)
        }
    }

    #[inline]
    pub fn map_capture_moves<F: FnMut(Move)>(&self, mut apply_move: F) {
        if self.side() == Side::WHITE {
            self.map_legal_moves_internal::<_, WHITE, true>(&mut apply_move)
        } else {
            self.map_legal_moves_internal::<_, BLACK, true>(&mut apply_move)
        }
    }

    pub fn map_legal_moves_internal<F: FnMut(Move), const COLOR: u8, const CAPTURE_ONLY: bool>(
        &self,
        apply_move: &mut F,
    ) {
        let attack_map = self.generate_attack_map(Side::from(COLOR).flipped());
        let king_square = self.king_square(Side::from(COLOR));
        let (bishop_pins, rook_pins) = self.generate_pin_masks(Side::from(COLOR));
        let checkers = if attack_map.get_bit(king_square) {
            self.generate_checkers_mask(Side::from(COLOR))
        } else {
            Bitboard::EMPTY
        };

        MoveGen::generate_king_moves::<_, COLOR, CAPTURE_ONLY>(
            self,
            attack_map,
            king_square,
            apply_move,
        );

        if checkers.is_empty() {
            if !CAPTURE_ONLY {
                MoveGen::generate_castle_moves::<_, COLOR>(
                    self,
                    attack_map,
                    king_square,
                    rook_pins,
                    apply_move,
                )
            }

            let push_map = !self.occupancy();
            let capture_map = self.occupancy_for_side(Side::from(COLOR).flipped());

            MoveGen::generate_pawn_moves::<_, COLOR, CAPTURE_ONLY>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { KNIGHT }, CAPTURE_ONLY>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { BISHOP }, CAPTURE_ONLY>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { ROOK }, CAPTURE_ONLY>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
                apply_move,
            );
        } else if (checkers & (checkers - 1)).is_empty() {
            let checker = checkers.ls1b_square();
            let push_map = Rays::get_ray(king_square, checker).exclude(checker);

            MoveGen::generate_pawn_moves::<_, COLOR, CAPTURE_ONLY>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { KNIGHT }, CAPTURE_ONLY>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { BISHOP }, CAPTURE_ONLY>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { ROOK }, CAPTURE_ONLY>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
                apply_move,
            );
        }
    }

    pub fn count_legal_moves<const COLOR: u8>(&self) -> u32 {
        let attack_map = self.generate_attack_map(Side::from(COLOR).flipped());
        let king_square = self.king_square(Side::from(COLOR));
        let (bishop_pins, rook_pins) = self.generate_pin_masks(Side::from(COLOR));
        let checkers = if attack_map.get_bit(king_square) {
            self.generate_checkers_mask(Side::from(COLOR))
        } else {
            Bitboard::EMPTY
        };

        let mut result = 0;

        result += MoveGen::count_king_moves::<COLOR>(
            self,
            attack_map,
            king_square,
        );

        if checkers.is_empty() {
            result += MoveGen::count_castle_moves::<COLOR>(
                self,
                attack_map,
                king_square,
                rook_pins,
            );

            let push_map = !self.occupancy();
            let capture_map = self.occupancy_for_side(Side::from(COLOR).flipped());

            result += MoveGen::count_pawn_moves::<COLOR>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { KNIGHT }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { BISHOP }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { ROOK }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            );
        } else if (checkers & (checkers - 1)).is_empty() {
            let checker = checkers.ls1b_square();
            let push_map = Rays::get_ray(king_square, checker).exclude(checker);

            result += MoveGen::count_pawn_moves::<COLOR>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { KNIGHT }>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { BISHOP }>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { ROOK }>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
            );
        };

        result
    }
}
