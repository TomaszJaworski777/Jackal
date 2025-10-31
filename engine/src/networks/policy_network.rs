use chess::{Attacks, Bitboard, ChessBoard, Move, MoveFlag, Piece, Side, Square};

use crate::networks::{inputs::{Standard768, Threats3072}, layers::{Accumulator, NetworkLayer, TransposedNetworkLayer}};

const INPUT_SIZE: usize = Standard768::input_size();
const HL_SIZE: usize = 512;

#[repr(C)]
#[derive(Debug)]
pub struct PolicyNetwork {
    l0: NetworkLayer<f32, INPUT_SIZE, HL_SIZE>,
    l1: TransposedNetworkLayer<f32, HL_SIZE, {1880 * 2}>
}

impl PolicyNetwork {
    pub fn create_base(&self, board: &ChessBoard) -> Accumulator<f32, HL_SIZE> {
        let mut result = *self.l0.biases();

        Standard768::map_inputs(board, |weight_idx| {
            for (i, weight) in result
                .values_mut()
                .iter_mut()
                .zip(self.l0.weights()[weight_idx].values())
            {
                *i += *weight;
            }
        });

        result
    }

    pub fn forward(&self, board: &ChessBoard, base: &Accumulator<f32, HL_SIZE>, mv: Move, see: bool, chess960: bool) -> f32 {
        let idx = map_move_to_index(board, mv, see, chess960);
        let weights = self.l1.weights()[idx];
        let mut result = self.l1.biases().values()[idx];

        for (&weight, &value) in weights.values().iter().zip(base.values()) {
            let acc = value.clamp(0.0, 1.0).powi(2);
            result += weight * acc;
        }

        result
    }
}

fn map_move_to_index(board: &ChessBoard, mv: Move, see: bool, chess960: bool) -> usize {
    let horizontal_mirror = if board.king_square(board.side()).get_file() > 3 { 7 } else { 0 };
    let good_see = (OFFSETS[64] + PROMOS) * usize::from(see);

    let idx = if mv.is_promotion() {
        let from_file = (mv.get_from_square() ^ horizontal_mirror).get_file();
        let to_file = (mv.get_to_square() ^ horizontal_mirror).get_file();
        let promo_id = 2 * from_file + to_file;

        OFFSETS[64] + 22 * (usize::from(mv.get_promotion_piece()) - usize::from(Piece::KNIGHT)) + usize::from(promo_id)
    } else {
        let flip = if board.side() == Side::WHITE { 0 } else { 56 };
        let from = usize::from(mv.get_from_square() ^ flip ^ horizontal_mirror);
        let to = if mv.is_castle() && !chess960 {
            let side = if u8::from(mv.get_from_square()) < 32 {
                Side::WHITE
            } else {
                Side::BLACK
            };
            let destination_square = if mv.get_flag() == MoveFlag::QUEEN_SIDE_CASTLE {
                Square::C1
            } else {
                Square::G1
            } + 56 * u8::from(side);

            usize::from(destination_square ^ flip ^ horizontal_mirror)
        } else {
            usize::from(mv.get_to_square() ^ flip ^ horizontal_mirror)
        };

        let below = ALL_DESTINATIONS[from] & ((1 << to) - 1);

        OFFSETS[from] + below.count_ones() as usize
    };

    good_see + idx
}

const PROMOS: usize = 4 * 22;

const OFFSETS: [usize; 65] = {
    let mut offsets = [0; 65];

    let mut current_offset = 0;
    let mut square_index = 0;

    while square_index < 64 {
        offsets[square_index] = current_offset;
        current_offset += ALL_DESTINATIONS[square_index].count_ones() as usize;
        square_index += 1;
    }

    offsets[64] = current_offset;

    offsets
};

pub const ALL_DESTINATIONS: [u64; 64] = {
    let mut result = [0u64; 64];
    let mut square_index = 0;
    while square_index < 64 {
        let square = Square::from_value(square_index as u8);

        let rank = square.get_rank() as usize;
        let file = square.get_file() as usize;

        let rooks = (Bitboard::RANK_1.get_value() << ((rank * 8) as u32)) ^ (Bitboard::FILE_A.get_value() << (file as u32));
        let bishops = DIAGONALS[file + rank].swap_bytes() ^ DIAGONALS[7 + file - rank];

        result[square_index] = rooks | bishops | Attacks::get_knight_attacks(Square::from_value(square_index as u8)).get_value() | Attacks::get_king_attacks(Square::from_value(square_index as u8)).get_value();

        square_index += 1;
    }

    result
};

pub const DIAGONALS: [u64; 15] = [
    0x0100_0000_0000_0000,
    0x0201_0000_0000_0000,
    0x0402_0100_0000_0000,
    0x0804_0201_0000_0000,
    0x1008_0402_0100_0000,
    0x2010_0804_0201_0000,
    0x4020_1008_0402_0100,
    0x8040_2010_0804_0201,
    0x0080_4020_1008_0402,
    0x0000_8040_2010_0804,
    0x0000_0080_4020_1008,
    0x0000_0000_8040_2010,
    0x0000_0000_0080_4020,
    0x0000_0000_0000_8040,
    0x0000_0000_0000_0080,
];