use crate::spear::{attacks::Rays, Bitboard, ChessBoard, Move};

pub struct MoveGen;
impl ChessBoard {
    #[inline]
    pub fn map_moves<F: FnMut(Move), const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        mut method: F,
    ) {
        Self::map_moves_internal::<F, false, STM_WHITE, NSTM_WHITE>(self, &mut method)
    }

    #[inline]
    pub fn map_captures<F: FnMut(Move), const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        mut method: F,
    ) {
        Self::map_moves_internal::<F, true, STM_WHITE, NSTM_WHITE>(self, &mut method)
    }

    fn map_moves_internal<
        F: FnMut(Move),
        const CAPTURE_ONLY: bool,
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
    >(
        &self,
        method: &mut F,
    ) {
        let attack_map = self.generate_attack_map::<STM_WHITE, NSTM_WHITE>();
        let king_square = self.get_king_square::<STM_WHITE>();
        let (diagonal_pins, ortographic_pins) = self.generate_pin_masks::<STM_WHITE, NSTM_WHITE>();
        let checkers = if self.is_square_attacked_with_attack_map(king_square, attack_map) {
            self.generate_checkers_mask::<STM_WHITE, NSTM_WHITE>()
        } else {
            Bitboard::EMPTY
        };

        MoveGen::generate_king_moves::<F, CAPTURE_ONLY, NSTM_WHITE>(
            self,
            attack_map,
            king_square,
            method,
        );

        if checkers.is_empty() {
            if !CAPTURE_ONLY {
                MoveGen::generate_castle_moves::<F, STM_WHITE>(
                    self,
                    attack_map,
                    king_square,
                    method,
                )
            }

            let push_map = !self.get_occupancy();
            let capture_map = self.get_occupancy_for_side::<NSTM_WHITE>();

            MoveGen::generate_pawn_moves::<F, STM_WHITE, NSTM_WHITE, CAPTURE_ONLY>(
                self,
                push_map,
                capture_map,
                diagonal_pins,
                ortographic_pins,
                method,
            );
            MoveGen::generate_piece_moves::<F, CAPTURE_ONLY, { MoveGen::KNIGHT }, STM_WHITE>(
                self,
                push_map,
                capture_map,
                diagonal_pins,
                ortographic_pins,
                method,
            );
            MoveGen::generate_piece_moves::<F, CAPTURE_ONLY, { MoveGen::BISHOP }, STM_WHITE>(
                self,
                push_map,
                capture_map,
                diagonal_pins,
                ortographic_pins,
                method,
            );
            MoveGen::generate_piece_moves::<F, CAPTURE_ONLY, { MoveGen::ROOK }, STM_WHITE>(
                self,
                push_map,
                capture_map,
                diagonal_pins,
                ortographic_pins,
                method,
            );
        } else if (checkers & (checkers - 1)).is_empty() {
            let checker = checkers.ls1b_square();
            let push_map = Rays::get_ray(king_square, checker).exclude(checker);

            MoveGen::generate_pawn_moves::<F, STM_WHITE, NSTM_WHITE, CAPTURE_ONLY>(
                self,
                push_map,
                checkers,
                diagonal_pins,
                ortographic_pins,
                method,
            );
            MoveGen::generate_piece_moves::<F, CAPTURE_ONLY, { MoveGen::KNIGHT }, STM_WHITE>(
                self,
                push_map,
                checkers,
                diagonal_pins,
                ortographic_pins,
                method,
            );
            MoveGen::generate_piece_moves::<F, CAPTURE_ONLY, { MoveGen::BISHOP }, STM_WHITE>(
                self,
                push_map,
                checkers,
                diagonal_pins,
                ortographic_pins,
                method,
            );
            MoveGen::generate_piece_moves::<F, CAPTURE_ONLY, { MoveGen::ROOK }, STM_WHITE>(
                self,
                push_map,
                checkers,
                diagonal_pins,
                ortographic_pins,
                method,
            );
        }
    }
}
