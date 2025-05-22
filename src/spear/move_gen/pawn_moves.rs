use crate::spear::{attacks::Attacks, Bitboard, ChessBoard, Move, MoveFlag, Piece, Square};

use super::MoveGen;

impl MoveGen {
    pub fn generate_pawn_moves<
        F: FnMut(Move),
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
        const CAPTURE_ONLY: bool,
    >(
        board: &ChessBoard,
        push_map: Bitboard,
        capture_map: Bitboard,
        diagonal_pins: Bitboard,
        ortographic_pins: Bitboard,
        method: &mut F,
    ) {
        let promotion_rank = if STM_WHITE {
            Bitboard::RANK_7
        } else {
            Bitboard::RANK_2
        };
        let double_push_rank = if STM_WHITE {
            Bitboard::RANK_2
        } else {
            Bitboard::RANK_7
        };
        let pawns = board.get_piece_mask_for_side::<STM_WHITE>(Piece::PAWN);

        let pushable_pawns = pawns & !diagonal_pins;
        let attack_pawns = pawns & !ortographic_pins;

        handle_pawn_captures::<F, STM_WHITE>(
            attack_pawns,
            capture_map,
            diagonal_pins,
            promotion_rank,
            method,
        );

        if board.en_passant_square() != Square::NULL {
            handle_en_passant::<F, STM_WHITE, NSTM_WHITE>(board, attack_pawns, method)
        }

        if CAPTURE_ONLY {
            return;
        }

        handle_pawn_pushes::<F, STM_WHITE>(
            board,
            pushable_pawns,
            push_map,
            ortographic_pins,
            promotion_rank,
            double_push_rank,
            method,
        );
    }
}

fn handle_pawn_pushes<F: FnMut(Move), const STM_WHITE: bool>(
    board: &ChessBoard,
    pushable_pawns: Bitboard,
    push_map: Bitboard,
    ortographic_pins: Bitboard,
    promotion_rank: Bitboard,
    double_push_rank: Bitboard,
    method: &mut F,
) {
    let vertical_pins = ortographic_pins & ortographic_pins.shift_left(8);
    let vertical_pins = vertical_pins | vertical_pins.shift_right(8);
    let pinned_pawns = pushable_pawns & vertical_pins;
    let pushable_pawns = (pushable_pawns & !ortographic_pins) | pinned_pawns;
    let promotion_pawns = pushable_pawns & promotion_rank;
    let double_pushable_pawns = pushable_pawns & double_push_rank;

    let empty_spaces = !board.get_occupancy();
    let single_shifted_push_map = if STM_WHITE {
        push_map.shift_right(8)
    } else {
        push_map.shift_left(8)
    };
    let double_shifted_push_map = if STM_WHITE {
        push_map.shift_right(16)
    } else {
        push_map.shift_left(16)
    };
    let single_shifted_empty = if STM_WHITE {
        empty_spaces.shift_right(8)
    } else {
        empty_spaces.shift_left(8)
    };

    let single_push_pawns = pushable_pawns & !promotion_pawns & single_shifted_push_map;
    let targets = if STM_WHITE {
        single_push_pawns.shift_left(8)
    } else {
        single_push_pawns.shift_right(8)
    };
    for (from_square, to_square) in single_push_pawns.into_iter().zip(targets) {
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::QUIET_MOVE,
        ))
    }

    let double_push_pawns = double_pushable_pawns & single_shifted_empty & double_shifted_push_map;
    let targets = if STM_WHITE {
        double_push_pawns.shift_left(16)
    } else {
        double_push_pawns.shift_right(16)
    };
    for (from_square, to_square) in double_push_pawns.into_iter().zip(targets) {
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::DOUBLE_PUSH,
        ))
    }

    let promotion_pawns = promotion_pawns & single_shifted_push_map;
    let targets = if STM_WHITE {
        promotion_pawns.shift_left(8)
    } else {
        promotion_pawns.shift_right(8)
    };
    for (from_square, to_square) in promotion_pawns.into_iter().zip(targets) {
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::KNIGHT_PROMOTION,
        ));
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::BISHOP_PROMOTION,
        ));
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::ROOK_PROMOTION,
        ));
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::QUEEN_PROMOTION,
        ));
    }
}

fn handle_pawn_captures<F: FnMut(Move), const STM_WHITE: bool>(
    attack_pawns: Bitboard,
    capture_map: Bitboard,
    diagonal_pins: Bitboard,
    promotion_rank: Bitboard,
    method: &mut F,
) {
    let shift_amount = if STM_WHITE { 7 } else { 9 };
    let left_pin = diagonal_pins & diagonal_pins.shift_left(shift_amount);
    let left_pin = left_pin | left_pin.shift_right(shift_amount);

    let shift_amount = if STM_WHITE { 9 } else { 7 };
    let right_pin = diagonal_pins & diagonal_pins.shift_left(shift_amount);
    let right_pin = right_pin | right_pin.shift_right(shift_amount);

    let left_shifted_captures = if STM_WHITE {
        capture_map.shift_right(7)
    } else {
        capture_map.shift_left(9)
    };
    let right_shifted_captures = if STM_WHITE {
        capture_map.shift_right(9)
    } else {
        capture_map.shift_left(7)
    };

    let left_attack_pawns =
        ((attack_pawns & !diagonal_pins) | (attack_pawns & left_pin)) & !Bitboard::FILE_A;
    let right_attack_pawns =
        ((attack_pawns & !diagonal_pins) | (attack_pawns & right_pin)) & !Bitboard::FILE_H;
    let left_promotion_pawns = left_attack_pawns & promotion_rank;
    let right_promotion_pawns = right_attack_pawns & promotion_rank;

    let left_capture_pawns = left_attack_pawns & !left_promotion_pawns & left_shifted_captures;
    let targets = if STM_WHITE {
        left_capture_pawns.shift_left(7)
    } else {
        left_capture_pawns.shift_right(9)
    };
    for (from_square, to_square) in left_capture_pawns.into_iter().zip(targets) {
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::CAPTURE,
        ))
    }

    let right_capture_pawns = right_attack_pawns & !right_promotion_pawns & right_shifted_captures;
    let targets = if STM_WHITE {
        right_capture_pawns.shift_left(9)
    } else {
        right_capture_pawns.shift_right(7)
    };
    for (from_square, to_square) in right_capture_pawns.into_iter().zip(targets) {
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::CAPTURE,
        ))
    }

    let left_promotion_pawns = left_promotion_pawns & left_shifted_captures;
    let targets = if STM_WHITE {
        left_promotion_pawns.shift_left(7)
    } else {
        left_promotion_pawns.shift_right(9)
    };
    for (from_square, to_square) in left_promotion_pawns.into_iter().zip(targets) {
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::KNIGHT_PROMOTION_CAPTURE,
        ));
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::BISHOP_PROMOTION_CAPTURE,
        ));
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::ROOK_PROMOTION_CAPTURE,
        ));
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::QUEEN_PROMOTION_CAPTURE,
        ));
    }

    let right_promotion_pawns = right_promotion_pawns & right_shifted_captures;
    let targets = if STM_WHITE {
        right_promotion_pawns.shift_left(9)
    } else {
        right_promotion_pawns.shift_right(7)
    };
    for (from_square, to_square) in right_promotion_pawns.into_iter().zip(targets) {
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::KNIGHT_PROMOTION_CAPTURE,
        ));
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::BISHOP_PROMOTION_CAPTURE,
        ));
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::ROOK_PROMOTION_CAPTURE,
        ));
        method(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::QUEEN_PROMOTION_CAPTURE,
        ));
    }
}

fn handle_en_passant<F: FnMut(Move), const STM_WHITE: bool, const NSTM_WHITE: bool>(
    board: &ChessBoard,
    attack_pawns: Bitboard,
    method: &mut F,
) {
    let pawns = Attacks::get_pawn_attacks_for_square::<NSTM_WHITE>(board.en_passant_square())
        & attack_pawns;

    pawns.map(|pawn_square| {
        let mut board_copy = *board;
        let new_mv =
            Move::from_squares(pawn_square, board.en_passant_square(), MoveFlag::EN_PASSANT);
        board_copy.make_move::<STM_WHITE, NSTM_WHITE>(new_mv);

        let king_square = board_copy.get_king_square::<STM_WHITE>();
        if !board_copy.is_square_attacked::<STM_WHITE, NSTM_WHITE>(king_square) {
            method(new_mv);
        }
    });
}
