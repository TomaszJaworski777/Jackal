use chess::{Bitboard, Piece, Rays, Side};

const STATE_INPUTS: usize = 768 * 6 * 2 * 2;

pub struct ThreatsExtended;
#[allow(unused)]
impl ThreatsExtended {
    pub const fn input_size() -> usize {
        STATE_INPUTS
    }

    pub fn map_inputs<F: FnMut(usize)>(board: &chess::ChessBoard, mut process_input: F) {
        let (mut diag_stm, mut ortho_stm) = board.generate_pin_masks(board.side());
        let (mut diag_nstm, mut ortho_nstm) = board.generate_pin_masks(board.side().flipped());

        let horizontal_mirror = if board.king_square(board.side()).file() > 3 {
            7
        } else {
            0
        };

        let flip = board.side() == Side::BLACK;

        let occ = board.occupancy();
        let mut side_offset = 0;
        for piece_color in if flip { [Side::BLACK, Side::WHITE] } else { [Side::WHITE, Side::BLACK] } {
            let defender_pin_mask = diag_stm | ortho_stm;
            let attack_pin_mask = diag_nstm | ortho_nstm;

            let occ_stm = board.occupancy_for_side(piece_color);
            let occ_nstm = board.occupancy_for_side(piece_color.flipped());

            for piece_idx in u8::from(Piece::PAWN)..=u8::from(Piece::KING) {
                let piece = Piece::from(piece_idx);
                let feat_idx = side_offset + 64 * (u8::from(piece) - u8::from(Piece::PAWN)) as usize;

                board.piece_mask_for_side(piece, piece_color).map(|square| {
                    let attack_pin_mask = attack_pin_mask & !Rays::get_ray(square, board.king_square(piece_color.flipped()));

                    let all_attackers = board.all_attackers_to_square(occ, square);
                    let attackers = all_attackers & occ_nstm & !attack_pin_mask;
                    let defenders = all_attackers & occ_stm & !defender_pin_mask;

                    let base = feat_idx + (usize::from(square) ^ horizontal_mirror ^ if flip { 56 } else { 0 });

                    let mut feat = 768 * calculate_state(board, piece, attackers, defenders);

                    if diag_stm.get_bit(square) {
                        feat += 768 * 6;
                    }

                    if ortho_stm.get_bit(square) {
                        feat += 768 * 6 * 2;
                    }

                    feat += base;

                    process_input(feat)
                });
            }

            side_offset += 384;
            (diag_stm, ortho_stm, diag_nstm, ortho_nstm) = (diag_nstm, ortho_nstm, diag_stm, ortho_stm);
        }
    }
}

const PIECE_VALUES: [usize; 6] = [100, 200, 300, 500, 650, 99999];
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

    if atk_cnt > 1 && lowest_defender == Piece::KING {
        return 0;
    }

    if atk_cnt > 1 && def_cnt < atk_cnt && v_victim + v_defender > v_attacker {
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