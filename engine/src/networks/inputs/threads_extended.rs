use chess::{Bitboard, Piece, Rays, Side};

const STATE_INPUTS: usize = 768 * 6 * 2 * 2;

pub struct ThreatsExtended;
#[allow(unused)]
impl ThreatsExtended {
    pub const fn input_size() -> usize {
        STATE_INPUTS
    }

    pub fn map_inputs<F: FnMut(usize)>(board: &chess::ChessBoard, mut process_input: F) {
        let (diag_stm, ortho_stm) = board.generate_pin_masks(board.side());
        let defender_pin_mask = diag_stm | ortho_stm;

        let (diag_nstm, ortho_nstm) = board.generate_pin_masks(board.side().flipped());
        let attack_pin_mask = diag_nstm | ortho_nstm;

        let horizontal_mirror = if board.king_square(board.side()).file() > 3 {
            7
        } else {
            0
        };

        let flip = board.side() == Side::BLACK;

        let occ = board.occupancy();
        occ.map(|square| {
            let piece = board.piece_on_square(square);
            let color = board.color_on_square(square);

            let attack_pin_mask = attack_pin_mask & !Rays::get_ray(square, board.king_square(board.side().flipped()));

            let all_attackers = board.all_attackers_to_square(occ, square);
            let attackers = all_attackers & board.occupancy_for_side(board.side().flipped()) & !attack_pin_mask;
            let defenders = all_attackers & board.occupancy_for_side(board.side()) & !defender_pin_mask;

            let piece_index = 64 * (u8::from(piece) - u8::from(Piece::PAWN)) as usize;
            let base = [384, 0][usize::from(color == board.side())] + piece_index + (usize::from(square) ^ horizontal_mirror ^ if flip { 56 } else { 0 });

            let mut feat = 768 * calculate_state(board, piece, attackers, defenders);

            if (if color == board.side() { diag_stm } else { diag_nstm }).get_bit(square) {
                feat += 768 * 6;
            }

            if (if color == board.side() { ortho_stm } else { ortho_nstm }).get_bit(square) {
                feat += 768 * 6 * 2;
            }

            feat += base;

            process_input(feat)
        });
    }
}

const PIECE_VALUES: [usize; 6] = [100, 300, 300, 500, 1000, 99999];
fn calculate_state(board: &chess::ChessBoard, victim: Piece, attackers: Bitboard, defenders: Bitboard) -> usize {
    let lowest_attacker = lowest_value_piece(board, attackers);
    let lowest_defender = lowest_value_piece(board, defenders);

    let atk_cnt = attackers.pop_count();
    let def_cnt = defenders.pop_count();

    if atk_cnt + def_cnt == 0 {
        return 5;
    }

    if def_cnt == 0 && atk_cnt > 0 {
        return 0; 
    }
    
    if atk_cnt == 0 && def_cnt > 0 {
        return 4;
    }

    let v_victim = PIECE_VALUES[u8::from(victim) as usize];
    let v_attacker = PIECE_VALUES[u8::from(lowest_attacker) as usize];
    let v_defender = PIECE_VALUES[u8::from(lowest_defender) as usize];
    
    if atk_cnt == 1 && def_cnt == 1 {
        if v_attacker < v_victim {
            return 1;
        } else if v_attacker > v_victim {
            return 3;
        } else {
            return 2;
        }
    }

    if atk_cnt > 1 && def_cnt == 1 && lowest_defender == Piece::KING {
        return 0;
    }

    if atk_cnt > 1 && def_cnt <= atk_cnt && v_victim + v_defender > v_attacker {
        return 1;
    }

    if atk_cnt > 1 && def_cnt == atk_cnt && v_victim + v_defender < v_attacker {
        return 3;
    }
    
    let diff = (def_cnt as i32) - (atk_cnt as i32);

    if diff < 0 {
        return 1;
    } else if diff == 0 {
        if v_attacker < v_victim {
             return 1;
        }
        return 2;
    } else {
        return 3
    }
}

fn lowest_value_piece(board: &chess::ChessBoard, mask: Bitboard) -> Piece {
    if mask.is_empty() {
        return Piece::NONE;
    }

    if (mask & board.piece_mask(Piece::PAWN)).is_not_empty() {
        Piece::PAWN
    } else if (mask & board.piece_mask(Piece::KNIGHT)).is_not_empty() {
        Piece::KNIGHT
    } else if (mask & board.piece_mask(Piece::BISHOP)).is_not_empty() {
        Piece::BISHOP
    } else if (mask & board.piece_mask(Piece::ROOK)).is_not_empty() {
        Piece::ROOK
    } else if (mask & board.piece_mask(Piece::QUEEN)).is_not_empty() {
        Piece::QUEEN
    } else if (mask & board.piece_mask(Piece::KING)).is_not_empty() {
        Piece::KING
    } else {
        Piece::NONE
    }
}