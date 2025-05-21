use crate::spear::{Attacks, Bitboard, ChessBoard, Move, MoveFlag, Piece, Side, Square};


pub struct SEE;
impl SEE {
    pub const PIECE_VALUES: [i32; 6] = [100, 300, 300, 500, 900, 0];

    pub fn static_exchange_evaluation<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        board: &ChessBoard,
        mv: Move,
        threshold: i32,
    ) -> bool {
        let from_square = mv.get_from_square();
        let to_square = mv.get_to_square();

        // Next victim is moved piece or promotion type
        let mut next_victim = if mv.is_promotion() {
            mv.get_promotion_piece()
        } else {
            board.get_piece_on_square(from_square)
        };

        // Balance is the value of the move minus threshold. Function
        // call takes care for Enpass, Promotion and Castling moves.
        let mut balance = estimate_move_value(&board, mv) - threshold;

        // Best case still fails to beat the threshold
        if balance < 0 {
            return false;
        }

        // Worst case is losing the moved piece
        balance -= SEE::PIECE_VALUES[next_victim.get_raw() as usize];

        // If the balance is positive even if losing the moved piece,
        // the exchange is guaranteed to beat the threshold.
        if balance >= 0 {
            return true;
        }

        // Grab sliders for updating revealed attackers
        let bishops = board.get_piece_mask(Piece::BISHOP) | board.get_piece_mask(Piece::QUEEN);
        let rooks = board.get_piece_mask(Piece::ROOK) | board.get_piece_mask(Piece::QUEEN);

        // Let occupied suppose that the move was actually made
        let mut occupied = board.get_occupancy();
        occupied = (occupied ^ from_square.get_bit()) | to_square.get_bit();
        if mv.is_en_passant() {
            occupied = occupied ^ board.en_passant_square().get_bit();
        }

        // Get all pieces which attack the target square. And with occupied
        // so that we do not let the same piece attack twice
        let mut attackers = (Attacks::get_knight_attacks_for_square(to_square)
            & board.get_piece_mask(Piece::KNIGHT))
            | (Attacks::get_king_attacks_for_square(to_square) & board.get_piece_mask(Piece::KING))
            | (Attacks::get_pawn_attacks_for_square::<false>(to_square)
                & board.get_piece_mask(Piece::PAWN)
                & board.get_occupancy_for_side::<true>())
            | (Attacks::get_pawn_attacks_for_square::<true>(to_square)
                & board.get_piece_mask(Piece::PAWN)
                & board.get_occupancy_for_side::<false>())
            | (Attacks::get_rook_attacks_for_square(to_square, occupied) & rooks)
            | (Attacks::get_bishop_attacks_for_square(to_square, occupied) & bishops);

        // Now our opponents turn to recapture
        let mut side_to_move = board.side_to_move().flipped();
        Self::see_internal::<NSTM_WHITE, STM_WHITE>(
            board,
            &mut next_victim,
            &mut balance,
            &mut occupied,
            &mut attackers,
            &mut side_to_move,
            to_square,
            bishops,
            rooks,
        );

        // Side to move after the loop loses
        board.side_to_move() != side_to_move
    }

    fn see_internal<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        board: &ChessBoard,
        next_victim: &mut Piece,
        balance: &mut i32,
        occupied: &mut Bitboard,
        attackers: &mut Bitboard,
        side_to_move: &mut Side,
        to_square: Square,
        bishops: Bitboard,
        rooks: Bitboard,
    ) {
        *side_to_move = if STM_WHITE { Side::WHITE } else { Side::BLACK };

        // If we have no more attackers left we lose
        let my_attackers = *attackers & board.get_occupancy_for_side::<STM_WHITE>();
        if my_attackers.is_empty() {
            return;
        }

        // Find our weakest piece to attack with
        for new_next_victim in Piece::PAWN.get_raw()..=Piece::KING.get_raw() {
            *next_victim = Piece::from_raw(new_next_victim);
            if (my_attackers & board.get_piece_mask(Piece::from_raw(new_next_victim)))
                .is_not_empty()
            {
                break;
            }
        }

        // Remove this attacker from the occupied
        *occupied = *occupied
            ^ (1u64
                << (my_attackers & board.get_piece_mask(*next_victim))
                    .ls1b_square()
                    .get_raw());

        // A diagonal move may reveal bishop or queen attackers
        if *next_victim == Piece::PAWN
            || *next_victim == Piece::BISHOP
            || *next_victim == Piece::QUEEN
        {
            *attackers |= Attacks::get_bishop_attacks_for_square(to_square, *occupied) & bishops;
        }

        // A vertical or horizontal move may reveal rook or queen attackers
        if *next_victim == Piece::ROOK || *next_victim == Piece::QUEEN {
            *attackers |= Attacks::get_rook_attacks_for_square(to_square, *occupied) & rooks;
        }

        // Make sure we did not add any already used attacks
        *attackers &= *occupied;

        // Swap the turn
        side_to_move.mut_flip();

        // Negamax the balance and add the value of the next victim
        *balance = -*balance - 1 - SEE::PIECE_VALUES[next_victim.get_raw() as usize];

        // If the balance is non negative after giving away our piece then we win
        if *balance >= 0 {
            // As a slide speed up for move legality checking, if our last attacking
            // piece is a king, and our opponent still has attackers, then we've
            // lost as the move we followed would be illegal
            if *next_victim == Piece::KING
                && (*attackers & board.get_occupancy_for_side::<STM_WHITE>()).is_not_empty()
            {
                side_to_move.mut_flip();
            }

            return;
        }

        Self::see_internal::<NSTM_WHITE, STM_WHITE>(
            board,
            next_victim,
            balance,
            occupied,
            attackers,
            side_to_move,
            to_square,
            bishops,
            rooks,
        );
    }
}

fn estimate_move_value(board: &ChessBoard, mv: Move) -> i32 {
    // Start with the value of the piece on the target square
    let target_piece = board.get_piece_on_square(mv.get_to_square());
    let mut value = if target_piece == Piece::NONE {
        0
    } else {
        SEE::PIECE_VALUES[target_piece.get_raw() as usize]
    };

    // Factor in the new piece's value and remove our promoted pawn
    if mv.is_promotion() {
        value +=
            SEE::PIECE_VALUES[mv.get_promotion_piece().get_raw() as usize] - SEE::PIECE_VALUES[0];
    }
    // Target square is encoded as empty for enpass moves
    else if mv.is_en_passant() {
        value = SEE::PIECE_VALUES[0];
    }
    // We encode Castle moves as KxR, so the initial step is wrong
    else if mv.get_flag() == MoveFlag::KING_SIDE_CASTLE
        || mv.get_flag() == MoveFlag::QUEEN_SIDE_CASTLE
    {
        value = 0;
    }

    return value;
}
