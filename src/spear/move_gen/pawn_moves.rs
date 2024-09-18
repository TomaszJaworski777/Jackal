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
        let promotion_rank = Bitboard::RANK_7 >> (board.side_to_move().get_raw() * 40) as u32;
        let double_push_rank = Bitboard::RANK_2 << (board.side_to_move().get_raw() * 40) as u32;
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
    let pinned_pawns = pushable_pawns & ortographic_pins;
    let not_pinned_pawns = pushable_pawns & !pinned_pawns;

    (not_pinned_pawns & !promotion_rank).map(|pawn_square| {
        let to_square = if STM_WHITE {
            pawn_square.shift_left(8)
        } else {
            pawn_square.shift_right(8)
        };
        if push_map.get_bit(to_square) {
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::QUIET_MOVE,
            ))
        }
    });

    pinned_pawns.map(|pawn_square| {
        let to_square = if STM_WHITE {
            pawn_square.shift_left(8)
        } else {
            pawn_square.shift_right(8)
        };
        if (push_map & ortographic_pins).get_bit(to_square) {
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::QUIET_MOVE,
            ))
        }
    });

    (not_pinned_pawns & promotion_rank).map(|pawn_square| {
        let to_square = if STM_WHITE {
            pawn_square.shift_left(8)
        } else {
            pawn_square.shift_right(8)
        };
        if push_map.get_bit(to_square) {
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::KNIGHT_PROMOTION,
            ));
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::BISHOP_PROMOTION,
            ));
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::ROOK_PROMOTION,
            ));
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::QUEEN_PROMOTION,
            ));
        }
    });

    (not_pinned_pawns & double_push_rank).map(|pawn_square| {
        let passing_square = if STM_WHITE {
            pawn_square.shift_left(8)
        } else {
            pawn_square.shift_right(8)
        };
        let to_square = if STM_WHITE {
            pawn_square.shift_left(16)
        } else {
            pawn_square.shift_right(16)
        };
        if (!board.get_occupancy()).get_bit(passing_square) && push_map.get_bit(to_square) {
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::DOUBLE_PUSH,
            ))
        }
    });

    (pinned_pawns & double_push_rank).map(|pawn_square| {
        let passing_square = if STM_WHITE {
            pawn_square.shift_left(8)
        } else {
            pawn_square.shift_right(8)
        };
        let to_square = if STM_WHITE {
            pawn_square.shift_left(16)
        } else {
            pawn_square.shift_right(16)
        };
        if (!board.get_occupancy()).get_bit(passing_square)
            && (push_map & ortographic_pins).get_bit(to_square)
        {
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::DOUBLE_PUSH,
            ))
        }
    });
}

fn handle_pawn_captures<F: FnMut(Move), const STM_WHITE: bool>(
    attack_pawns: Bitboard,
    capture_map: Bitboard,
    diagonal_pins: Bitboard,
    promotion_rank: Bitboard,
    method: &mut F,
) {
    let pinned_pawns = attack_pawns & diagonal_pins;
    let not_pinned_pawns = attack_pawns & !pinned_pawns;

    (not_pinned_pawns & !promotion_rank).map(|pawn_square| {
        let attacks = Attacks::get_pawn_attacks_for_square::<STM_WHITE>(pawn_square) & capture_map;
        attacks.map(|to_square| {
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::CAPTURE,
            ))
        })
    });

    (pinned_pawns & !promotion_rank).map(|pawn_square| {
        let attacks = Attacks::get_pawn_attacks_for_square::<STM_WHITE>(pawn_square)
            & capture_map
            & diagonal_pins;
        attacks.map(|to_square| {
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::CAPTURE,
            ))
        })
    });

    (not_pinned_pawns & promotion_rank).map(|pawn_square| {
        let attacks = Attacks::get_pawn_attacks_for_square::<STM_WHITE>(pawn_square) & capture_map;
        attacks.map(|to_square| {
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::KNIGHT_PROMOTION_CAPTURE,
            ));
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::BISHOP_PROMOTION_CAPTURE,
            ));
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::ROOK_PROMOTION_CAPTURE,
            ));
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::QUEEN_PROMOTION_CAPTURE,
            ));
        })
    });

    (pinned_pawns & promotion_rank).map(|pawn_square| {
        let attacks = Attacks::get_pawn_attacks_for_square::<STM_WHITE>(pawn_square)
            & capture_map
            & diagonal_pins;
        attacks.map(|to_square| {
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::KNIGHT_PROMOTION_CAPTURE,
            ));
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::BISHOP_PROMOTION_CAPTURE,
            ));
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::ROOK_PROMOTION_CAPTURE,
            ));
            method(Move::from_squares(
                pawn_square,
                to_square,
                MoveFlag::QUEEN_PROMOTION_CAPTURE,
            ));
        })
    });
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
