use crate::{
    move_gen::generate_moves::{MoveGen, WHITE},
    Attacks, Bitboard, ChessBoard, Move, MoveFlag, Piece, Side, Square,
};

impl MoveGen {
    pub fn generate_pawn_moves<F: FnMut(Move), const COLOR: u8, const CAPTURE_ONLY: bool>(
        board: &ChessBoard,
        push_map: Bitboard,
        capture_map: Bitboard,
        bishop_pins: Bitboard,
        rook_pins: Bitboard,
        apply_move: &mut F,
    ) {
        let pawns = board.piece_mask_for_side(Piece::PAWN, Side::from(COLOR));

        handle_pawn_captures::<_, COLOR>(
            pawns & !rook_pins,
            capture_map,
            bishop_pins,
            apply_move,
        );

        if board.en_passant_square() != Square::NULL {
            handle_en_passant::<_, COLOR>(board, pawns, apply_move);
        }

        if CAPTURE_ONLY {
            return;
        }

        handle_pawn_pushes::<_, COLOR>(
            board,
            pawns & !bishop_pins,
            push_map,
            rook_pins,
            apply_move,
        );
    }
}

fn handle_pawn_pushes<F: FnMut(Move), const COLOR: u8>(
    board: &ChessBoard,
    pawns: Bitboard,
    push_map: Bitboard,
    rook_pins: Bitboard,
    apply_move: &mut F,
) {
    let vertical_pin = rook_pins & (rook_pins << 8);
    let vertical_pin = vertical_pin | rook_pins >> 8;

    let promotion_rank = if COLOR == WHITE {
        Bitboard::RANK_7
    } else {
        Bitboard::RANK_2
    };
    let double_push_rank = if COLOR == WHITE {
        Bitboard::RANK_2
    } else {
        Bitboard::RANK_7
    };

    let moveable_pawns = pawns & !promotion_rank;
    let moveable_pawns = (moveable_pawns & !rook_pins) | (moveable_pawns & vertical_pin);

    let single_push_map = if COLOR == WHITE {
        push_map >> 8
    } else {
        push_map << 8
    };

    let mut single_push_pawns = moveable_pawns & single_push_map;
    let mut targets = if COLOR == WHITE {
        single_push_pawns << 8
    } else {
        single_push_pawns >> 8
    };
    while single_push_pawns.is_not_empty() {
        apply_move(Move::from_squares(
            single_push_pawns.pop_ls1b_square(),
            targets.pop_ls1b_square(),
            MoveFlag::QUIET_MOVE,
        ))
    }

    let double_push_map = if COLOR == WHITE {
        push_map >> 16
    } else {
        push_map << 16
    };
    let empty_squares_shifted_map = if COLOR == WHITE {
        !board.occupancy() >> 8
    } else {
        !board.occupancy() << 8
    };

    let mut double_push_pawns =
        moveable_pawns & double_push_rank & empty_squares_shifted_map & double_push_map;
    let mut targets = if COLOR == WHITE {
        double_push_pawns << 16
    } else {
        double_push_pawns >> 16
    };
    while double_push_pawns.is_not_empty() {
        apply_move(Move::from_squares(
            double_push_pawns.pop_ls1b_square(),
            targets.pop_ls1b_square(),
            MoveFlag::DOUBLE_PUSH,
        ))
    }

    let promotion_pawns = pawns & promotion_rank & !rook_pins;
    let targets = if COLOR == WHITE {
        promotion_pawns << 8
    } else {
        promotion_pawns >> 8
    } & push_map;
    targets.map(|to_square| {
        let from_square = if COLOR == WHITE {
            to_square >> 8
        } else {
            to_square << 8
        };
        apply_move(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::QUEEN_PROMOTION,
        ));
        apply_move(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::ROOK_PROMOTION,
        ));
        apply_move(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::BISHOP_PROMOTION,
        ));
        apply_move(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::KNIGHT_PROMOTION,
        ));
    });
}

fn handle_pawn_captures<F: FnMut(Move), const COLOR: u8>(
    mut pawns: Bitboard,
    capture_map: Bitboard,
    bishop_pins: Bitboard,
    apply_move: &mut F,
) {
    let promotion_rank = if COLOR == WHITE {
        Bitboard::RANK_7
    } else {
        Bitboard::RANK_2
    };
    let promotion_pawns = pawns & promotion_rank;
    pawns &= !promotion_rank;

    (pawns & !bishop_pins).map(|from_square| {
        let attacks = Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map;
        attacks.map(|to_square| {
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::CAPTURE,
            ))
        });
    });

    (pawns & bishop_pins).map(|from_square| {
        let attacks =
            Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map & bishop_pins;
        attacks.map(|to_square| {
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::CAPTURE,
            ))
        });
    });

    (promotion_pawns & !bishop_pins).map(|from_square| {
        let attacks = Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map;
        attacks.map(|to_square| {
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::QUEEN_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::ROOK_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::BISHOP_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::KNIGHT_PROMOTION_CAPTURE,
            ));
        });
    });

    (promotion_pawns & bishop_pins).map(|from_square| {
        let attacks =
            Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map & bishop_pins;
        attacks.map(|to_square| {
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::QUEEN_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::ROOK_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::BISHOP_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::KNIGHT_PROMOTION_CAPTURE,
            ));
        });
    });
}

fn handle_en_passant<F: FnMut(Move), const COLOR: u8>(
    board: &ChessBoard,
    mut pawns: Bitboard,
    apply_move: &mut F,
) {
    let en_passant_square = board.en_passant_square();
    pawns &= Attacks::get_pawn_attacks(en_passant_square, Side::from(COLOR).flipped());

    pawns.map(|from_square| {
        let mut board = *board;
        let mv = Move::from_squares(from_square, en_passant_square, MoveFlag::EN_PASSANT);
        board.make_move_no_mask(mv);

        if !board.is_square_attacked(board.king_square(Side::from(COLOR)), Side::from(COLOR)) {
            apply_move(mv)
        }
    });
}
