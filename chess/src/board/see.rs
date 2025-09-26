use crate::{Attacks, ChessBoard, Move, Piece};

const SEE_VALUES: [i32; 6] = [100, 450, 450, 650, 1250, 0];

fn see_value(piece: Piece) -> i32 {
    SEE_VALUES[usize::from(piece)]
}

impl ChessBoard {
    pub fn see_value(piece: Piece) -> i32 {
        see_value(piece)
    }

    pub fn see(&self, mv: Move, threshold: i32) -> bool {

        // Unpack move information
        let from = mv.get_from_square();
        let to = mv.get_to_square();

        let mut next_victim = if mv.is_promotion() {
            mv.get_promotion_piece()
        } else {
            self.piece_on_square(from)
        };

        // Balance is the value of the move minus threshold. Function
        // call takes care for Enpass, Promotion and Castling moves.
        let mut balance = self.move_value(mv) - threshold;

        // Best case still fails to beat the threshold
        if balance < 0 {
            return false;
        }

        // Worst case is losing the moved piece
        balance -= see_value(next_victim);

        // If the balance is positive even if losing the moved piece,
        // the exchange is guaranteed to beat the threshold.
        if balance >= 0 {
            return true;
        }

        // Grab sliders for updating revealed attackers
        let bishops = self.piece_mask(Piece::BISHOP) | self.piece_mask(Piece::QUEEN);
        let rooks = self.piece_mask(Piece::ROOK) | self.piece_mask(Piece::QUEEN);

        // Let occupied suppose that the move was actually made
        let mut occupancy = self.occupancy().exclude(from).include(to);
        if mv.is_en_passant() {
            occupancy = occupancy.exclude(self.en_passant_square())
        }

        // Get all pieces which attack the target square. And with occupied
        // so that we do not let the same piece attack twice
        let mut attackers = self.all_attackers_to_square(occupancy, to) & occupancy;

        // Now our opponents turn to recapture
        let mut side = self.side().flipped();

        loop {

            // If we have no more attackers left we lose
            let m_atackers = attackers & self.occupancy_for_side(side);
            if m_atackers.is_empty() {
                break;
            }

            // Find our weakest piece to attack with
            for new_next_victim_idx in u8::from(Piece::PAWN)..=u8::from(Piece::KING) {
                next_victim = Piece::from(new_next_victim_idx);
                if (m_atackers & self.piece_mask(next_victim)).is_not_empty() {
                    break;
                }
            }

            // Remove this attacker from the occupied
            occupancy = occupancy.exclude((m_atackers & self.piece_mask(next_victim)).ls1b_square());

            // A diagonal move may reveal bishop or queen attackers
            if next_victim == Piece::PAWN || next_victim == Piece::BISHOP || next_victim == Piece::QUEEN {
                attackers |= Attacks::get_bishop_attacks(to, occupancy) & bishops;
            }

            // A vertical or horizontal move may reveal rook or queen attackers
            if next_victim == Piece::ROOK || next_victim == Piece::QUEEN {
                attackers |= Attacks::get_rook_attacks(to, occupancy) & rooks;
            }

            // Make sure we did not add any already used attacks
            attackers &= occupancy;

            // Swap the turn
            side.flip();

            // Negamax the balance and add the value of the next victim
            balance = -balance - 1 - see_value(next_victim);

            // If the balance is non negative after giving away our piece then we win
            if balance >= 0 {

                // As a slide speed up for move legality checking, if our last attacking
                // piece is a king, and our opponent still has attackers, then we've
                // lost as the move we followed would be illegal
                if next_victim == Piece::KING && (attackers & self.occupancy_for_side(side.flipped())).is_not_empty() {
                    side.flip();
                }

                break;
            }
        }

        // Side to move after the loop loses
        self.side() != side
    }

    fn move_value(&self, mv: Move) -> i32 {
        let mut value = 0;

        // Start with the value of the piece on the target square
        if mv.is_capture() & !mv.is_en_passant() {
            value += see_value(self.piece_on_square(mv.get_to_square()));
        }

        // Factor in the new piece's value and remove our promoted pawn
        if mv.is_promotion() {
            value += see_value(mv.get_promotion_piece()) - see_value(Piece::PAWN);

        // Target square is encoded as empty for enpass moves
        } else if mv.is_en_passant() {
            value = see_value(Piece::PAWN);

        // We encode Castle moves as KxR, so the initial step is wrong
        } else if mv.is_castle() {
            value = 0;
        }

        value
    }
}
