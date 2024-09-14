use crate::spear::{base_structures::Move, CastleRights, ChessBoard, MoveFlag, Piece, Square};

use super::chess_board_state::PHASE_VALUES;

impl ChessBoard {
    #[inline]
    pub fn make_move<const STM_WHITE: bool, const NSTM_WHITE: bool>(&mut self, mv: Move) {
        self.make_move_move_flag::<STM_WHITE, NSTM_WHITE>(
            mv,
            mv.get_from_square(),
            mv.get_to_square(),
        )
    }

    #[inline]
    fn make_move_move_flag<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &mut self,
        mv: Move,
        from_square: Square,
        to_square: Square,
    ) {
        match mv.get_flag() {
            MoveFlag::QUIET_MOVE => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::QUIET_MOVE }>(mv, from_square, to_square),
            MoveFlag::DOUBLE_PUSH => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::DOUBLE_PUSH }>(mv, from_square, to_square),
            MoveFlag::KING_SIDE_CASTLE => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::KING_SIDE_CASTLE }>(mv, from_square, to_square),
            MoveFlag::QUEEN_SIDE_CASTLE => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::QUEEN_SIDE_CASTLE }>(mv, from_square, to_square),
            MoveFlag::CAPTURE => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::CAPTURE }>(mv, from_square, to_square),
            MoveFlag::EN_PASSANT => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::EN_PASSANT }>(mv, from_square, to_square),
            MoveFlag::KNIGHT_PROMOTION => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::KNIGHT_PROMOTION }>(mv, from_square, to_square),
            MoveFlag::BISHOP_PROMOTION => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::BISHOP_PROMOTION }>(mv, from_square, to_square),
            MoveFlag::ROOK_PROMOTION => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::ROOK_PROMOTION }>(mv, from_square, to_square),
            MoveFlag::QUEEN_PROMOTION => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::QUEEN_PROMOTION }>(mv, from_square, to_square),
            MoveFlag::KNIGHT_PROMOTION_CAPTURE => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::KNIGHT_PROMOTION_CAPTURE }>(mv, from_square, to_square),
            MoveFlag::BISHOP_PROMOTION_CAPTURE => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::BISHOP_PROMOTION_CAPTURE }>(mv, from_square, to_square),
            MoveFlag::ROOK_PROMOTION_CAPTURE => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::ROOK_PROMOTION_CAPTURE }>(mv, from_square, to_square),
            MoveFlag::QUEEN_PROMOTION_CAPTURE => self.make_move_moved_piece::<STM_WHITE, NSTM_WHITE, { MoveFlag::QUEEN_PROMOTION_CAPTURE }>(mv, from_square, to_square),
            _ => unreachable!()
        }
    }

    #[inline]
    fn make_move_moved_piece<
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
        const MOVE_FLAG: u16,
    >(
        &mut self,
        mv: Move,
        from_square: Square,
        to_square: Square,
    ) {
        let moved_piece = self.get_piece_on_square(from_square);
        match moved_piece {
            Piece::PAWN => self
                .make_move_captured_piece::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, { PAWN }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::KNIGHT => self
                .make_move_captured_piece::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, { KNIGHT }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::BISHOP => self
                .make_move_captured_piece::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, { BISHOP }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::ROOK => self
                .make_move_captured_piece::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, { ROOK }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::QUEEN => self
                .make_move_captured_piece::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, { QUEEN }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::KING => self
                .make_move_captured_piece::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, { KING }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::NONE => self
                .make_move_captured_piece::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, { NONE }>(
                    mv,
                    from_square,
                    to_square,
                ),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn make_move_captured_piece<
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
        const MOVE_FLAG: u16,
        const MOVED_PIECE: u8,
    >(
        &mut self,
        mv: Move,
        from_square: Square,
        to_square: Square,
    ) {
        if MOVE_FLAG & MoveFlag::CAPTURE == 0 {
            self.make_move_internal::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, MOVED_PIECE, { NONE }>(
                mv,
                from_square,
                to_square,
            );
            return;
        }

        let captured_piece = self.get_piece_on_square(to_square);
        match captured_piece {
            Piece::PAWN => self
                .make_move_internal::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, MOVED_PIECE, { PAWN }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::KNIGHT => self
                .make_move_internal::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, MOVED_PIECE, { KNIGHT }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::BISHOP => self
                .make_move_internal::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, MOVED_PIECE, { BISHOP }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::ROOK => self
                .make_move_internal::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, MOVED_PIECE, { ROOK }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::QUEEN => self
                .make_move_internal::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, MOVED_PIECE, { QUEEN }>(
                    mv,
                    from_square,
                    to_square,
                ),
            Piece::NONE => self
                .make_move_internal::<STM_WHITE, NSTM_WHITE, MOVE_FLAG, MOVED_PIECE, { NONE }>(
                    mv,
                    from_square,
                    to_square,
                ),
            _ => {
                self.draw_board();
                println!("{mv}");
                unreachable!()
            }
        }
    }

    fn make_move_internal<
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
        const MOVE_FLAG: u16,
        const MOVED_PIECE: u8,
        const CAPTURED_PIECE: u8,
    >(
        &mut self,
        mv: Move,
        from_square: Square,
        to_square: Square,
    ) {
        if CAPTURED_PIECE != NONE {
            self.remove_piece_on_square::<NSTM_WHITE>(to_square, Piece::from_raw(CAPTURED_PIECE));
            *self.state.get_phase_mut() -= PHASE_VALUES[CAPTURED_PIECE as usize];
        }

        self.remove_piece_on_square::<STM_WHITE>(from_square, Piece::from_raw(MOVED_PIECE));
        if MOVE_FLAG < MoveFlag::KNIGHT_PROMOTION {
            self.set_piece_on_square::<STM_WHITE>(to_square, Piece::from_raw(MOVED_PIECE));
        }

        if MOVED_PIECE == PAWN || MOVE_FLAG & MoveFlag::CAPTURE > 0 {
            *self.state.get_half_move_counter_mut() = 0;
        } else {
            *self.state.get_half_move_counter_mut() += 1;
        }

        let mut castle_rights = self.castle_rights().get_raw();
        castle_rights &= !(CastleRights::ROOK_MASKS[from_square.get_raw() as usize]
            | CastleRights::ROOK_MASKS[to_square.get_raw() as usize]);
        *self.state.get_castle_rights_mut() = CastleRights::from_raw(castle_rights);

        *self.state.get_en_passant_mut() = Square::NULL;

        match MOVE_FLAG {
            MoveFlag::DOUBLE_PUSH => {
                *self.state.get_en_passant_mut() = to_square ^ 8;
            }
            MoveFlag::KING_SIDE_CASTLE | MoveFlag::QUEEN_SIDE_CASTLE => {
                let king_side = usize::from(MOVE_FLAG == MoveFlag::KING_SIDE_CASTLE);
                let side_flip = 56 * usize::from(!STM_WHITE) as u8;
                let rook_from_square = side_flip + CastleRights::ROOK_POSITIONS[king_side];
                let rook_to_square = side_flip + [3, 5][king_side];
                self.remove_piece_on_square::<STM_WHITE>(
                    Square::from_raw(rook_from_square),
                    Piece::ROOK,
                );
                self.set_piece_on_square::<STM_WHITE>(
                    Square::from_raw(rook_to_square),
                    Piece::ROOK,
                );
            }
            MoveFlag::EN_PASSANT => {
                self.remove_piece_on_square::<NSTM_WHITE>(to_square ^ 8, Piece::PAWN)
            }
            MoveFlag::KNIGHT_PROMOTION.. => {
                let promotion_piece = mv.get_promotion_piece();
                self.set_piece_on_square::<STM_WHITE>(to_square, promotion_piece);
                *self.state.get_phase_mut() -= PHASE_VALUES[PAWN as usize];
                *self.state.get_phase_mut() += PHASE_VALUES[promotion_piece.get_raw() as usize];
            }
            _ => {}
        }

        self.state.get_side_to_move_mut().mut_flip();
    }
}

const PAWN: u8 = 0;
const KNIGHT: u8 = 1;
const BISHOP: u8 = 2;
const ROOK: u8 = 3;
const QUEEN: u8 = 4;
const KING: u8 = 5;
const NONE: u8 = 6;
