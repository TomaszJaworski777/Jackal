use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Write},
    time::Instant,
};

use bullet::format::ChessBoard;
use spear::{ChessBoardPacked, Piece};

use crate::bullet_convert_display::BulletConvertDisplay;

#[derive(PartialEq)]
pub enum DataConvertionMode {
    Full,
    NoDraws,
}

pub struct BulletConverter;
impl BulletConverter {
    pub fn convert(input_path: &str, output_path: &str, mode: DataConvertionMode) {
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
        let mut mode_filter = 0;
        let mut mate_scores = 0;

        let mut white_wins = 0;
        let mut white_draws = 0;
        let mut white_loses = 0;

        while reader.read_exact(&mut buffer).is_ok() {
            if timer.elapsed().as_secs_f32() > 1.0 {
                BulletConvertDisplay::print_report(
                    entries_processed,
                    entry_count,
                    white_wins,
                    white_draws,
                    white_loses,
                    unfiltered,
                    mode_filter,
                    mate_scores,
                );
                timer = Instant::now();
            }

            let position: ChessBoardPacked = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
            entries_processed += 1;

            if mode == DataConvertionMode::NoDraws && position.get_result() == 0 {
                mode_filter += 1;
                continue;
            }

            let score = position.get_white_perspective_score();
            if score <= 0.0 || score >= 1.0 {
                mate_scores += 1;
                continue;
            }

            //rest of the filters

            let score = -(400.0 * (1.0 / score - 1.0).ln()) as i16;
            let result = (position.get_result() + 1) as f32 / 2.0;

            let board = spear::ChessBoard::from_board_pack(&position);
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
