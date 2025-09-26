use crate::{ChessBoard, Move, MoveFlag, Piece, Side, Square};

impl ChessBoard {
    pub fn make_move_no_mask(&mut self, mv: Move) {
        let mask = self.castle_rights().get_castle_mask();
        self.make_move(mv, &mask);
    }

    pub fn make_move(&mut self, mv: Move, castle_mask: &[u8; 64]) {
        let from = mv.get_from_square();
        let to = mv.get_to_square();

        if self.side() == Side::WHITE {
            self.make_move_move_flag::<0>(mv, from, to, castle_mask);
        } else {
            self.make_move_move_flag::<1>(mv, from, to, castle_mask);
        }
    }

    pub(crate) fn make_move_templated<const COLOR: u8>(
        &mut self,
        mv: Move,
        castle_mask: &[u8; 64],
    ) {
        let from = mv.get_from_square();
        let to = mv.get_to_square();

        self.make_move_move_flag::<COLOR>(mv, from, to, castle_mask);
    }

    #[inline]
    fn make_move_move_flag<const COLOR: u8>(
        &mut self,
        mv: Move,
        from_square: Square,
        to_square: Square,
        mask: &[u8; 64],
    ) {
        match mv.get_flag() {
            MoveFlag::QUIET_MOVE => self.make_move_moved_piece::<COLOR, { MoveFlag::QUIET_MOVE }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            MoveFlag::DOUBLE_PUSH => self
                .make_move_moved_piece::<COLOR, { MoveFlag::DOUBLE_PUSH }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::KING_SIDE_CASTLE => self
                .make_move_moved_piece::<COLOR, { MoveFlag::KING_SIDE_CASTLE }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::QUEEN_SIDE_CASTLE => self
                .make_move_moved_piece::<COLOR, { MoveFlag::QUEEN_SIDE_CASTLE }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::CAPTURE => self.make_move_moved_piece::<COLOR, { MoveFlag::CAPTURE }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            MoveFlag::EN_PASSANT => self.make_move_moved_piece::<COLOR, { MoveFlag::EN_PASSANT }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            MoveFlag::KNIGHT_PROMOTION => self
                .make_move_moved_piece::<COLOR, { MoveFlag::KNIGHT_PROMOTION }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::BISHOP_PROMOTION => self
                .make_move_moved_piece::<COLOR, { MoveFlag::BISHOP_PROMOTION }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::ROOK_PROMOTION => self
                .make_move_moved_piece::<COLOR, { MoveFlag::ROOK_PROMOTION }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::QUEEN_PROMOTION => self
                .make_move_moved_piece::<COLOR, { MoveFlag::QUEEN_PROMOTION }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::KNIGHT_PROMOTION_CAPTURE => self
                .make_move_moved_piece::<COLOR, { MoveFlag::KNIGHT_PROMOTION_CAPTURE }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::BISHOP_PROMOTION_CAPTURE => self
                .make_move_moved_piece::<COLOR, { MoveFlag::BISHOP_PROMOTION_CAPTURE }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::ROOK_PROMOTION_CAPTURE => self
                .make_move_moved_piece::<COLOR, { MoveFlag::ROOK_PROMOTION_CAPTURE }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            MoveFlag::QUEEN_PROMOTION_CAPTURE => self
                .make_move_moved_piece::<COLOR, { MoveFlag::QUEEN_PROMOTION_CAPTURE }>(
                    mv,
                    from_square,
                    to_square,
                    mask,
                ),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn make_move_moved_piece<const COLOR: u8, const MOVE_FLAG: u16>(
        &mut self,
        mv: Move,
        from_square: Square,
        to_square: Square,
        mask: &[u8; 64],
    ) {
        let moved_piece = self.piece_on_square(from_square);
        match moved_piece {
            Piece::PAWN => self.make_move_captured_piece::<COLOR, MOVE_FLAG, { PAWN }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::KNIGHT => self.make_move_captured_piece::<COLOR, MOVE_FLAG, { KNIGHT }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::BISHOP => self.make_move_captured_piece::<COLOR, MOVE_FLAG, { BISHOP }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::ROOK => self.make_move_captured_piece::<COLOR, MOVE_FLAG, { ROOK }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::QUEEN => self.make_move_captured_piece::<COLOR, MOVE_FLAG, { QUEEN }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::KING => self.make_move_captured_piece::<COLOR, MOVE_FLAG, { KING }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::NONE => self.make_move_captured_piece::<COLOR, MOVE_FLAG, { NONE }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn make_move_captured_piece<const COLOR: u8, const MOVE_FLAG: u16, const MOVED_PIECE: u8>(
        &mut self,
        mv: Move,
        from_square: Square,
        to_square: Square,
        mask: &[u8; 64],
    ) {
        if MOVE_FLAG & MoveFlag::CAPTURE == 0 {
            self.make_move_internal::<COLOR, MOVE_FLAG, MOVED_PIECE, { NONE }>(
                mv,
                from_square,
                to_square,
                mask,
            );
            return;
        }

        let captured_piece = self.piece_on_square(to_square);
        match captured_piece {
            Piece::PAWN => self.make_move_internal::<COLOR, MOVE_FLAG, MOVED_PIECE, { PAWN }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::KNIGHT => self.make_move_internal::<COLOR, MOVE_FLAG, MOVED_PIECE, { KNIGHT }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::BISHOP => self.make_move_internal::<COLOR, MOVE_FLAG, MOVED_PIECE, { BISHOP }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::ROOK => self.make_move_internal::<COLOR, MOVE_FLAG, MOVED_PIECE, { ROOK }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::QUEEN => self.make_move_internal::<COLOR, MOVE_FLAG, MOVED_PIECE, { QUEEN }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            Piece::NONE => self.make_move_internal::<COLOR, MOVE_FLAG, MOVED_PIECE, { NONE }>(
                mv,
                from_square,
                to_square,
                mask,
            ),
            _ => {
                self.draw_board();
                println!("{}", mv.to_string(true));
                unreachable!()
            }
        }
    }

    fn make_move_internal<
        const COLOR: u8,
        const MOVE_FLAG: u16,
        const MOVED_PIECE: u8,
        const CAPTURED_PIECE: u8,
    >(
        &mut self,
        mv: Move,
        from_square: Square,
        to_square: Square,
        mask: &[u8; 64],
    ) {
        if CAPTURED_PIECE != NONE {
            self.remove_piece_on_square(
                to_square,
                Piece::from(CAPTURED_PIECE),
                Side::from(1 - COLOR),
            );
        }

        self.remove_piece_on_square(from_square, Piece::from(MOVED_PIECE), Side::from(COLOR));

        if MOVE_FLAG < MoveFlag::KNIGHT_PROMOTION
            && MOVE_FLAG != MoveFlag::KING_SIDE_CASTLE
            && MOVE_FLAG != MoveFlag::QUEEN_SIDE_CASTLE
        {
            self.set_piece_on_square(to_square, Piece::from(MOVED_PIECE), Side::from(COLOR));
        }

        if MOVED_PIECE == PAWN || MOVE_FLAG & MoveFlag::CAPTURE > 0 {
            self.half_moves = 0;
        } else {
            self.half_moves += 1;
        }

        let mut castle_rights = u8::from(self.castle_rights());
        castle_rights &= !(mask[usize::from(from_square)] | mask[usize::from(to_square)]);
        self.castle_rights.set_rights(castle_rights);

        self.en_passant_square = Square::NULL;

        let side_flip = usize::from(COLOR) * 56;
        match MOVE_FLAG {
            MoveFlag::DOUBLE_PUSH => {
                self.en_passant_square = to_square ^ 8;
            }
            MoveFlag::QUEEN_SIDE_CASTLE => {
                self.remove_piece_on_square(
                    self.castle_rights().rook_square(usize::from(COLOR) * 2),
                    Piece::ROOK,
                    Side::from(COLOR),
                );
                self.set_piece_on_square(
                    Square::from(side_flip as u8 + 3),
                    Piece::ROOK,
                    Side::from(COLOR),
                );
                self.set_piece_on_square(
                    Square::from(side_flip as u8 + 2),
                    Piece::KING,
                    Side::from(COLOR),
                );
            }
            MoveFlag::KING_SIDE_CASTLE => {
                self.remove_piece_on_square(
                    self.castle_rights().rook_square(usize::from(COLOR) * 2 + 1),
                    Piece::ROOK,
                    Side::from(COLOR),
                );
                self.set_piece_on_square(
                    Square::from(side_flip as u8 + 5),
                    Piece::ROOK,
                    Side::from(COLOR),
                );
                self.set_piece_on_square(
                    Square::from(side_flip as u8 + 6),
                    Piece::KING,
                    Side::from(COLOR),
                );
            }
            MoveFlag::EN_PASSANT => {
                self.remove_piece_on_square(to_square ^ 8, Piece::PAWN, Side::from(1 - COLOR))
            }
            MoveFlag::KNIGHT_PROMOTION.. => {
                let promotion_piece = mv.get_promotion_piece();
                self.set_piece_on_square(to_square, promotion_piece, Side::from(COLOR));
            }
            _ => {}
        }

        self.side.flip();
    }
}

const PAWN: u8 = 0;
const KNIGHT: u8 = 1;
const BISHOP: u8 = 2;
const ROOK: u8 = 3;
const QUEEN: u8 = 4;
const KING: u8 = 5;
const NONE: u8 = 6;
