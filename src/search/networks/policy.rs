use crate::spear::{ChessBoard, KingAttacks, KnightAttacks, Move, Piece, Side};

use crate::{Bitboard, Square, SEE};

use super::layer::TransposedNetworkLayer;
use super::{Accumulator, NetworkLayer};

#[allow(non_upper_case_globals)]
pub static PolicyNetwork: PolicyNetwork = unsafe {
    std::mem::transmute(*include_bytes!(
        "../../../resources/networks/policy_007-tdp1024see_10.network"
    ))
};

const HL_SIZE: usize = 1024;
const QA: i16 = 255;
const QB: i16 = 64;

#[repr(C)]
pub struct PolicyNetwork {
    l1: NetworkLayer<i16, {768 * 4}, HL_SIZE>,
    l2: TransposedNetworkLayer<i16, { HL_SIZE }, { 1880 * 2 }>,
}

impl PolicyNetwork {
    pub fn create_base<const STM_WHITE: bool, const NSTM_WHITE: bool>( &self, board: &ChessBoard ) -> Accumulator<i16, { HL_SIZE }> {
        let mut l1_out = *self.l1.biases();

        map_policy_inputs::<_, STM_WHITE, NSTM_WHITE>(board, |weight_index| {
            for (i, weight) in l1_out
                .values_mut()
                .iter_mut()
                .zip(self.l1.weights()[weight_index].values())
            {
                *i += *weight;
            }
        });

        l1_out

        //let mut out: Accumulator<i16, { HL_SIZE / 2 }> = Accumulator::default();

        // for (i, (&first, &second)) in out
        //     .values_mut()
        //     .iter_mut()
        //     .zip(l1_out.values().iter().take(HL_SIZE / 2).zip(l1_out.values().iter().skip(HL_SIZE / 2)))
        // {
        //     let first = i32::from(first).clamp(0, i32::from(QA));
        //     let second = i32::from(second).clamp(0, i32::from(QA));
        //     *i = ((first * second) / i32::from(QA)) as i16;
        // }

        // out
    }

    pub fn forward<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        board: &ChessBoard,
        base: &Accumulator<i16, { HL_SIZE }>,
        mv: Move,
    ) -> f32 {
        let idx = map_move_to_index::<STM_WHITE, NSTM_WHITE>(board, mv);
        let weights = self.l2.weights()[idx];
        let mut result = 0;

        for (&weight, &value) in weights.values().iter().zip(base.values()) {
            let acc = i32::from(value).clamp(0, i32::from(QA)).pow(2);
            result += i32::from(weight) * i32::from(acc);
        }

        (result as f32 / f32::from(QA) + f32::from(self.l2.biases().values()[idx])) / f32::from(QA * QB)
    }
}

fn map_policy_inputs<F: FnMut(usize), const STM_WHITE: bool, const NSTM_WHITE: bool>(
    board: &ChessBoard,
    mut method: F,
) {
    let horizontal_mirror = if board.get_king_square::<STM_WHITE>().get_file() > 3 {
        7
    } else {
        0
    };

    let flip = board.side_to_move() == Side::BLACK;

    let mut threats = board.generate_attack_map::<STM_WHITE, NSTM_WHITE>();
    let mut defences = board.generate_attack_map::<NSTM_WHITE, STM_WHITE>();

    if flip {
        threats = threats.flip();
        defences = defences.flip();
    }

    for piece in Piece::PAWN.get_raw()..=Piece::KING.get_raw() {
        let piece_index = 64 * (piece - Piece::PAWN.get_raw()) as usize;

        let mut stm_bitboard =
            board.get_piece_mask_for_side::<STM_WHITE>(Piece::from_raw(piece));
        let mut nstm_bitboard =
            board.get_piece_mask_for_side::<NSTM_WHITE>(Piece::from_raw(piece));

        if flip {
            stm_bitboard = stm_bitboard.flip();
            nstm_bitboard = nstm_bitboard.flip();
        }

        stm_bitboard.map(|square| {
            let mut feat = piece_index + (square.get_raw() as usize ^ horizontal_mirror);

            if threats.get_bit(square) {
                feat += 768;
            }

            if defences.get_bit(square) {
                feat += 768 * 2;
            }

            method(feat)
        });

        nstm_bitboard.map(|square| {
            let mut feat = 384 + piece_index + (square.get_raw() as usize ^ horizontal_mirror);

            if threats.get_bit(square) {
                feat += 768;
            }

            if defences.get_bit(square) {
                feat += 768 * 2;
            }

            method(feat)
        });
    }
}

fn map_move_to_index<const STM_WHITE: bool, const NSTM_WHITE: bool>(board: &ChessBoard, mv: Move) -> usize {
    let horizontal_mirror = if board.get_king_square::<STM_WHITE>().get_file() > 3 { 7 } else { 0 };
    let good_see = (OFFSETS[64] + PROMOS) * usize::from(SEE::static_exchange_evaluation::<STM_WHITE, NSTM_WHITE>(board, mv, -108,));

    let idx = if mv.is_promotion() {
        let from_file = (mv.get_from_square() ^ horizontal_mirror).get_file();
        let to_file = (mv.get_to_square() ^ horizontal_mirror).get_file();
        let promo_id = 2 * from_file + to_file;

        OFFSETS[64] + 22 * usize::from(mv.get_promotion_piece().get_raw() - Piece::KNIGHT.get_raw()) + usize::from(promo_id)
    } else {
        let flip = if board.side_to_move() == Side::WHITE { 0 } else { 56 };
        let from = usize::from(mv.get_from_square() ^ flip ^ horizontal_mirror);
        let to = usize::from(mv.get_to_square() ^ flip ^ horizontal_mirror);

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
        let square = Square::from_raw(square_index as u8);

        let rank = square.get_rank() as usize;
        let file = square.get_file() as usize;

        let rooks = (Bitboard::RANK_1.get_raw() << ((rank * 8) as u32)) ^ (Bitboard::FILE_A.get_raw() << (file as u32));
        let bishops = DIAGONALS[file + rank].swap_bytes() ^ DIAGONALS[7 + file - rank];

        result[square_index] = rooks | bishops | KnightAttacks::ATTACK_TABLE[square_index].get_raw() | KingAttacks::ATTACK_TABLE[square_index].get_raw();

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