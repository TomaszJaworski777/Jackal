use chess::{Attacks, Bitboard, ChessBoard, Move, MoveFlag, Side, Square};

use crate::networks::{inputs::Threats3072, layers::{Accumulator, NetworkLayer, TransposedNetworkLayer}};

const INPUT_SIZE: usize = Threats3072::input_size();
const HL_SIZE: usize = 8192;

const QA: i16 = 128;
const QB: i16 = 128;

#[repr(C)]
#[derive(Debug)]
pub struct PolicyNetwork {
    l0: NetworkLayer<i8, INPUT_SIZE, HL_SIZE>,
    l1: TransposedNetworkLayer<i8, { HL_SIZE / 2 }, NUM_MOVES_INDICES>
}

impl PolicyNetwork {
    pub fn create_base(&self, board: &ChessBoard) -> Accumulator<i16, { HL_SIZE / 2 }> {
        let mut inputs: Accumulator<i16, HL_SIZE> = Accumulator::default();

        for (i, &bias) in inputs.values_mut().iter_mut().zip(self.l0.biases().values()) {
            *i = i16::from(bias)
        }

        Threats3072::map_inputs(board, |weight_idx| {
            for (i, &weight) in inputs
                .values_mut()
                .iter_mut()
                .zip(self.l0.weights()[weight_idx].values())
            {
                *i += i16::from(weight);
            }
        });

        let mut result = Accumulator::default();

        for (i, (&a, &b)) in result.values_mut().iter_mut().zip(inputs.values().iter().take(HL_SIZE / 2).zip(inputs.values().iter().skip(HL_SIZE / 2))) {
            let a = i32::from(a).clamp(0, i32::from(QA));
            let b = i32::from(b).clamp(0, i32::from(QA));
            *i = ((a * b) / i32::from(QA)) as i16;
        }

        result
    }

    pub fn forward(&self, board: &ChessBoard, base: &Accumulator<i16, { HL_SIZE / 2 }>, mv: Move, see: bool, chess960: bool) -> f32 {
        let idx = map_move_to_index(board, mv, see, chess960);
        let weights = self.l1.weights()[idx];

        let mut result = 0;
        for (&weight, &neuron) in weights.values().iter().zip(base.values().iter()) {
            result += i32::from(weight) * i32::from(neuron);
        }

        (result as f32 / f32::from(QA) + f32::from(self.l1.biases().values()[idx])) / f32::from(QB)
    }
}

//Even though output mapping is literally copy pasted from monty, 
//the neural network weights are fully unique, so will still say that policy is 100% original.
pub const NUM_MOVES_INDICES: usize = 2 * FROM_TO;
pub const FROM_TO: usize = OFFSETS[5][64] + PROMOS + 2 + 8;
pub const PROMOS: usize = 4 * 22;

fn map_move_to_index(board: &ChessBoard, mv: Move, see: bool, _chess960: bool) -> usize {
    let hm = if board.king_square(board.side()).file() > 3 { 7 } else { 0 };
    let flip = hm ^ (if board.side() == Side::BLACK { 56 } else { 0 });

    let src = usize::from(mv.from_square() ^ flip);
    let dst = usize::from(mv.to_square() ^ flip);

    let good_see = usize::from(see);

    let idx = if mv.is_promotion() {
        let ffile = src % 8;
        let tfile = dst % 8;
        let promo_id = 2 * ffile + tfile;

        OFFSETS[5][64] + (PROMOS / 4) * (usize::from(mv.promotion_piece()) - 1) + promo_id
    } else if mv.flag() == MoveFlag::QUEEN_SIDE_CASTLE || mv.flag() == MoveFlag::KING_SIDE_CASTLE {
        let is_ks = usize::from(mv.flag() == MoveFlag::KING_SIDE_CASTLE);
        let is_hm = usize::from(hm == 0);
        OFFSETS[5][64] + PROMOS + (is_ks ^ is_hm)
    } else if mv.flag() == MoveFlag::DOUBLE_PUSH {
        OFFSETS[5][64] + PROMOS + 2 + (src % 8)
    } else {
        let pc = u8::from(board.piece_on_square(mv.from_square())) as usize;
        let below = DESTINATIONS[src][pc] & ((1 << dst) - 1);

        OFFSETS[pc][src] + below.count_ones() as usize
    };

    FROM_TO * good_see + idx
}

macro_rules! init {
    (|$sq:ident, $size:literal | $($rest:tt)+) => {{
        let mut $sq = 0;
        let mut res = [{$($rest)+}; $size];
        while $sq < $size {
            res[$sq] = {$($rest)+};
            $sq += 1;
        }
        res
    }};
}

const OFFSETS: [[usize; 65]; 6] = {
    let mut offsets = [[0; 65]; 6];

    let mut curr = 0;

    let mut pc = 0;
    while pc < 6 {
        let mut sq = 0;

        while sq < 64 {
            offsets[pc][sq] = curr;
            curr += DESTINATIONS[sq][pc].count_ones() as usize;
            sq += 1;
        }

        offsets[pc][64] = curr;

        pc += 1;
    }

    offsets
};

const DESTINATIONS: [[u64; 6]; 64] = init!(|sq, 64| [
    PAWN[sq],
    Attacks::get_knight_attacks(Square::from_value(sq as u8)).get_value(),
    bishop(sq),
    rook(sq),
    queen(sq),
    Attacks::get_king_attacks(Square::from_value(sq as u8)).get_value()
]);

const PAWN: [u64; 64] = init!(|sq, 64| {
    let bit = 1 << sq;
    ((bit & !Bitboard::FILE_A.get_value()) << 7) | (bit << 8) | ((bit & !Bitboard::FILE_H.get_value()) << 9)
});

const fn bishop(sq: usize) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;

    DIAGONALS[file + rank].swap_bytes() ^ DIAGONALS[7 + file - rank]
}

const fn rook(sq: usize) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;

    (0xFF << (rank * 8)) ^ (Bitboard::FILE_A.shift_left(file as u32)).get_value()
}

const fn queen(sq: usize) -> u64 {
    bishop(sq) | rook(sq)
}

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