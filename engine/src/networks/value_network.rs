use chess::ChessBoard;

use crate::{networks::{inputs::Threats3072, layers::{Accumulator, NetworkLayer, TransposedNetworkLayer}}, WDLScore};

const INPUT_SIZE: usize = Threats3072::input_size();
const L1_SIZE: usize = 3072;
const NUM_OUTPUT_BUCKETS: usize = 8;

const QA: i16 = 255;
const QB: i16 = 64;

#[repr(C)]
#[repr(align(64))]
#[derive(Debug)]
pub struct ValueNetwork {
    l0: NetworkLayer<i16, INPUT_SIZE, L1_SIZE>,
    l1: TransposedNetworkLayer<i16, L1_SIZE, { 3 * NUM_OUTPUT_BUCKETS }>
}

impl ValueNetwork {
    pub fn forward(&self, board: &ChessBoard) -> WDLScore {
        let mut l0_out = *self.l0.biases();

        Threats3072::map_inputs(board, |input_index| {
            for (bias, weight) in l0_out.values_mut().iter_mut().zip(self.l0.weights()[input_index].values()) {
                *bias += *weight
            }
        });

        let mut out = Accumulator::<i32, 3>::default();

        let bucket_idx = {
            let divisor = 32usize.div_ceil(NUM_OUTPUT_BUCKETS);
            (board.occupancy().pop_count() as usize - 2) / divisor
        } * 3;

        for (idx, output) in out.values_mut().iter_mut().enumerate() {
            let weights = self.l1.weights()[bucket_idx + idx];

            for (&weight, &l0_neuron) in weights.values().iter().zip(l0_out.values()) {
                *output += screlu(l0_neuron) * i32::from(weight);
            }
        }

        let mut win_chance = (out.values()[2] as f64 / f64::from(QA)
            + f64::from(self.l1.biases().values()[bucket_idx + 2]))
            / f64::from(QA * QB);
        let mut draw_chance = (out.values()[1] as f64 / f64::from(QA)
            + f64::from(self.l1.biases().values()[bucket_idx + 1]))
            / f64::from(QA * QB);
        let mut loss_chance = (out.values()[0] as f64 / f64::from(QA)
            + f64::from(self.l1.biases().values()[bucket_idx + 0]))
            / f64::from(QA * QB);

        let max = win_chance.max(draw_chance).max(loss_chance);

        win_chance = (win_chance - max).exp();
        draw_chance = (draw_chance - max).exp();
        loss_chance = (loss_chance - max).exp();

        let sum = win_chance + draw_chance + loss_chance;

        WDLScore::new(win_chance / sum, draw_chance / sum)
    }
}

fn screlu(x: i16) -> i32 {
    i32::from(x).clamp(0, i32::from(QA)).pow(2)
}