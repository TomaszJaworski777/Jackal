use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Write},
    time::Instant,
};

use jackal::{Piece, PolicyPacked};
use bullet::{default::formats::montyformat::chess::Position, game::formats::montyformat::chess::Move};

use super::PolicyConvertDisplay;

#[repr(C)]
#[repr(align(64))]
#[derive(Clone, Copy)]
pub struct DecompressedData {
    pub pos: Position,
    pub moves: [(u16, u16); 108],
    pub num: usize,
}

pub struct PolicyConvert;
impl PolicyConvert {
    pub fn convert(input_path: &str, output_path: &str) {
        let input_file = File::open(input_path).expect("Cannot open input file");
        let input_meta = input_file.metadata().expect("Cannot obtain file metadata");
        let mut reader = BufReader::new(input_file);

        let mut output_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_path)
            .expect("Cannot open output file");

        let decompressed_size = std::mem::size_of::<DecompressedData>();
        let mut buffer = [0u8; std::mem::size_of::<PolicyPacked>()];
        let entry_count = input_meta.len() / buffer.len() as u64;

        let mut timer = Instant::now();
        let mut entries_processed = 0;
        let mut unfiltered = 0;

        while reader.read_exact(&mut buffer).is_ok() {
            if timer.elapsed().as_secs_f32() > 1.0 {
                PolicyConvertDisplay::print_report(entries_processed, entry_count, unfiltered);
                timer = Instant::now();
            }

            let policy_pack: PolicyPacked = unsafe { std::mem::transmute(buffer) };

            let board = jackal::ChessBoard::from_policy_pack(&policy_pack);

            let mut bb = [0u64; 8];
            bb[0] = board.get_occupancy_for_side::<true>().get_raw();
            bb[1] = board.get_occupancy_for_side::<false>().get_raw();
            for idx in 0..6 {
                bb[idx + 2] = board.get_piece_mask(Piece::from_raw(idx as u8)).get_raw();
            }

            let stm = board.side_to_move() == jackal::Side::BLACK;

            let enp_sq = board.en_passant_square().get_raw();
            let castle_rights = board.castle_rights().get_raw();
            let half_move = board.half_move_counter();

            let pos = Position::from_raw(bb, stm, enp_sq, castle_rights, half_move, 0);

            let mut moves = [(0u16, 0u16); 108];
            for (idx, mv_pack) in policy_pack.moves().into_iter().enumerate() {
                let from = u16::from(mv_pack.mv.get_from_square().get_raw());
                let to = u16::from(mv_pack.mv.get_to_square().get_raw());
                let flag = u16::from(mv_pack.mv.get_flag());

                moves[idx] = (u16::from(Move::new(from, to, flag)), mv_pack.visits);
            }

            let decompressed = DecompressedData {
                pos,
                moves,
                num: usize::from(policy_pack.move_count()),
            };

            entries_processed += 1;

            let chess_board_bytes = unsafe {
                std::slice::from_raw_parts(
                    (&decompressed as *const DecompressedData) as *const u8,
                    decompressed_size,
                )
            };

            output_file
                .write_all(&chess_board_bytes)
                .expect("Couldnt write to output file");
            unfiltered += 1;
        }
    }
}
