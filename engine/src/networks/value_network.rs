use chess::ChessBoard;

use crate::{networks::{inputs::Threats3072, layers::{Accumulator, NetworkLayer, TransposedNetworkLayer}}, WDLScore};

const INPUT_SIZE: usize = Threats3072::input_size();
const HL_SIZE: usize = 2048;

const QA: i16 = 128;
const QB: i16 = 1024;

#[repr(C)]
#[repr(align(64))]
#[derive(Debug)]
pub struct ValueNetwork {
    l0: NetworkLayer<i8, INPUT_SIZE, HL_SIZE>,
    l1: TransposedNetworkLayer<i16, { HL_SIZE / 2 }, 16>,
    l2: NetworkLayer<f32, 16, 128>,
    l3: NetworkLayer<f32, 128, 3>
}

impl ValueNetwork {
    pub fn forward(&self, board: &ChessBoard) -> WDLScore {
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

        let mut l0_out: Accumulator<i16, { HL_SIZE / 2 }> = Accumulator::default();

        for (i, (&a, &b)) in l0_out.values_mut().iter_mut().zip(inputs.values().iter().take(HL_SIZE / 2).zip(inputs.values().iter().skip(HL_SIZE / 2))) {
            let a = a.clamp(0, QA);
            let b = b.clamp(0, QA);
            *i = a * b;
        }

        let mut fwd = [0; 16];

        for (f, row) in fwd.iter_mut().zip(self.l1.weights().iter()) {
            for (&a, &w) in l0_out.values().iter().zip(row.values().iter()) {
                *f += i32::from(a) * i32::from(w);
            }
        }

        let mut l1_out: Accumulator<f32, 16> = Accumulator::default();

        for (r, (&f, &b)) in l1_out.values_mut().iter_mut().zip(fwd.iter().zip(self.l2.biases().values().iter())) {
            *r = (f as f32 / f32::from(QA * QA) + f32::from(b)) / f32::from(QB);
        }

        let mut l2_out = *self.l2.biases();
        for (i, d) in l1_out.values().iter().zip(self.l2.weights().iter()) {
            l2_out.madd(i.clamp(0.0, 1.0).powi(2), d);
        }

        let mut out = *self.l3.biases();
        for (i, d) in l2_out.values().iter().zip(self.l3.weights().iter()) {
            out.madd(i.clamp(0.0, 1.0).powi(2), d);
        }

        let mut win_chance = out.values()[2] as f64;
        let mut draw_chance = out.values()[1] as f64;
        let mut loss_chance = out.values()[0] as f64;

        let max = win_chance.max(draw_chance).max(loss_chance);

        win_chance = (win_chance - max).exp();
        draw_chance = (draw_chance - max).exp();
        loss_chance = (loss_chance - max).exp();

        let sum = win_chance + draw_chance + loss_chance;

        WDLScore::new(win_chance / sum, draw_chance / sum)
    }
}