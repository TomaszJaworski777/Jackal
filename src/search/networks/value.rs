use crate::{search::networks::{layer::TransposedNetworkLayer, Accumulator, NetworkLayer}, spear::{ChessBoard, Piece, Side}};

use super::{QA, QB};

const NUM_OUTPUT_BUCKETS: usize = 8;

#[allow(non_upper_case_globals)]
pub static ValueNetwork: ValueNetwork = unsafe {
    std::mem::transmute(*include_bytes!(
        "../../../resources/networks/v600cos3072WDL-TD-OB-007b-Q.network"
    ))
};

#[repr(C)]
#[repr(align(64))]
pub struct ValueNetwork {
    l1: NetworkLayer<i16, { 768 * 4 }, 3072>,
    l2: TransposedNetworkLayer<i16, 3072, { 3 * NUM_OUTPUT_BUCKETS }>,
}

impl ValueNetwork {
    pub fn forward<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        board: &ChessBoard,
    ) -> (f32, f32, f32) {
        let mut l1_out = *self.l1.biases();

        Self::map_value_inputs::<_, STM_WHITE, NSTM_WHITE>(board, |weight_index| {
            for (i, weight) in l1_out
                .values_mut()
                .iter_mut()
                .zip(&self.l1.weights()[weight_index].vals)
            {
                *i += *weight;
            }
        });

        let mut out = Accumulator::<i32, 3>::default();

        let output_bucket = {
            let divisor = 32usize.div_ceil(NUM_OUTPUT_BUCKETS);
            (board.get_occupancy().pop_count() as usize - 2) / divisor
        } * 3;

        for (idx, output) in out.values_mut().iter_mut().enumerate() {
            let weights = self.l2.weights()[output_bucket + idx];

            for (&weight, &value) in weights.values().iter().zip(l1_out.values()) {
                let act = i32::from(value).clamp(0, i32::from(QA)).pow(2);
                *output += act * i32::from(weight);
            }
        }

        let mut win_chance = (out.values()[2] as f32 / f32::from(QA)
            + f32::from(self.l2.biases().values()[output_bucket + 2]))
            / f32::from(QA * QB);
        let mut draw_chance = (out.values()[1] as f32 / f32::from(QA)
            + f32::from(self.l2.biases().values()[output_bucket + 1]))
            / f32::from(QA * QB);
        let mut loss_chance = (out.values()[0] as f32 / f32::from(QA)
            + f32::from(self.l2.biases().values()[output_bucket + 0]))
            / f32::from(QA * QB);

        let max = win_chance.max(draw_chance).max(loss_chance);

        win_chance = (win_chance - max).exp();
        draw_chance = (draw_chance - max).exp();
        loss_chance = (loss_chance - max).exp();

        let sum = win_chance + draw_chance + loss_chance;

        (win_chance / sum, draw_chance / sum, loss_chance / sum)
    }

    fn map_value_inputs<F: FnMut(usize), const STM_WHITE: bool, const NSTM_WHITE: bool>(
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
}
