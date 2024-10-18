use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Write},
    time::Instant,
};

use bullet::format::ChessBoard;
use spear::{ChessBoardPacked, Move, Piece, Side};

use crate::value::ValueConvertDisplay;

pub struct ValueConverter;
impl ValueConverter {
    pub fn convert(input_path: &str, output_path: &str) {
        let input_file = File::open(input_path).expect("Cannot open input file");
        let input_meta = input_file.metadata().expect("Cannot obtain file metadata");
        let mut reader = BufReader::new(input_file);

        let mut output_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_path)
            .expect("Cannot open output file");

        let entry_size = std::mem::size_of::<ChessBoardPacked>();
        let bullet_board_size = std::mem::size_of::<ChessBoard>();
        let entry_count = input_meta.len() / entry_size as u64;
        let mut buffer = vec![0u8; entry_size];

        let mut timer = Instant::now();
        let mut entries_processed = 0;
        let mut unfiltered = 0;
        let mut mate_scores: u64 = 0;
        let mut material_advantage = 0;
        let mut draw = 0;

        let mut white_wins = 0;
        let mut white_draws = 0;
        let mut white_loses = 0;

        while reader.read_exact(&mut buffer).is_ok() {
            if timer.elapsed().as_secs_f32() > 1.0 {
                ValueConvertDisplay::print_report(
                    entries_processed,
                    entry_count,
                    white_wins,
                    white_draws,
                    white_loses,
                    unfiltered,
                    mate_scores,
                    material_advantage,
                    draw,
                );
                timer = Instant::now();
            }

            let position: ChessBoardPacked = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
            entries_processed += 1;

            let score = position.get_white_perspective_score();
            if score <= 0.0 || score >= 1.0 {
                mate_scores += 1;
                continue;
            }

            let board = spear::ChessBoard::from_board_pack(&position);
            let result = position.get_result();
            let material_score = calculate_material(&board);
            if ((result == 1 && material_score >= 0) || (result == -1 && material_score <= 0))
                && false
            {
                let material_score = if board.side_to_move() == Side::WHITE {
                    qsearch::<true, false>(&board, -30000, 30000, 0)
                } else {
                    -qsearch::<false, true>(&board, -30000, 30000, 0)
                };

                if (result == 1 && material_score >= 0) || (result == -1 && material_score <= 0) {
                    material_advantage += 1;
                    continue;
                }
            }

            if result == 0 && false {
                draw += 1;
                continue;
            }

            //rest of the filters

            let score = -(400.0 * (1.0 / score - 1.0).ln()) as i16;
            let result = (result + 1) as f32 / 2.0;

            let bbs = [
                board.get_occupancy_for_side::<true>().get_raw(),
                board.get_occupancy_for_side::<false>().get_raw(),
                board.get_piece_mask(Piece::PAWN).get_raw(),
                board.get_piece_mask(Piece::KNIGHT).get_raw(),
                board.get_piece_mask(Piece::BISHOP).get_raw(),
                board.get_piece_mask(Piece::ROOK).get_raw(),
                board.get_piece_mask(Piece::QUEEN).get_raw(),
                board.get_piece_mask(Piece::KING).get_raw(),
            ];

            let bullet_board = ChessBoard::from_raw(
                bbs,
                position.get_side_to_move().get_raw() as usize,
                score,
                result,
            )
            .expect("Couldnt create a bullet board");

            let chess_board_bytes = unsafe {
                std::slice::from_raw_parts(
                    (&bullet_board as *const ChessBoard) as *const u8,
                    bullet_board_size,
                )
            };

            match position.get_result() {
                -1 => white_loses += 1,
                0 => white_draws += 1,
                1 => white_wins += 1,
                _ => unreachable!(),
            }

            output_file
                .write_all(&chess_board_bytes)
                .expect("Couldnt write to output file");
            unfiltered += 1;
        }
    }
}

fn qsearch<const STM_WHITE: bool, const NSTM_WHITE: bool>(
    board: &spear::ChessBoard,
    mut alpha: i32,
    beta: i32,
    depth: u8,
) -> i32 {
    if board.is_insufficient_material() || board.half_move_counter() >= 100 {
        return 0;
    }

    let evaluation = if STM_WHITE {
        calculate_material(board)
    } else {
        -calculate_material(board)
    };

    if depth > 6 {
        return evaluation;
    }

    if evaluation >= beta {
        return beta;
    }

    if evaluation > alpha {
        alpha = evaluation;
    }

    let mut move_list = Vec::new();
    board.map_captures::<_, STM_WHITE, NSTM_WHITE>(|mv| move_list.push(mv));
    move_list.sort_by(|a, b| get_move_value(board, *b).cmp(&get_move_value(board, *a)));

    for mv_index in 0..move_list.len() {
        let mv = move_list[mv_index];
        if mv == Move::NULL {
            continue;
        }

        let mut board_copy = board.clone();
        board_copy.make_move::<STM_WHITE, NSTM_WHITE>(mv);

        let score = -qsearch::<NSTM_WHITE, STM_WHITE>(&board_copy, -beta, -alpha, depth + 1);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

#[inline]
fn get_move_value(position: &spear::ChessBoard, mv: Move) -> i32 {
    let mut result: i32 = 0;

    if mv.is_capture() {
        let target_piece = position.get_piece_on_square(mv.get_to_square());
        let moving_piece = position.get_piece_on_square(mv.get_from_square());
        result += ((target_piece.get_raw() + 1) as i32 * 100) - (moving_piece.get_raw() + 1) as i32;
    }
    if mv.is_promotion() {
        result += ((mv.get_promotion_piece().get_raw() + 1) as i32) * 100;
    }

    return result;
}

fn calculate_material(board: &spear::ChessBoard) -> i32 {
    const PIECE_VALUES: [i32; 5] = [100, 300, 300, 500, 900];
    let mut result = 0;

    for side in Side::WHITE.get_raw()..=Side::BLACK.get_raw() {
        for piece in Piece::PAWN.get_raw()..=Piece::QUEEN.get_raw() {
            let piece_mask = if side == Side::WHITE.get_raw() {
                board.get_piece_mask_for_side::<true>(Piece::from_raw(piece))
            } else {
                board.get_piece_mask_for_side::<false>(Piece::from_raw(piece))
            };
            result += piece_mask.pop_count() as i32 * PIECE_VALUES[piece as usize];
        }
        result = -result;
    }

    result
}
