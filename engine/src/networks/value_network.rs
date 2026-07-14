use chess::ChessBoard;

use crate::{
    networks::{
        inputs::ThreatsExtended,
        layers::{Accumulator, NetworkLayer, TransposedNetworkLayer},
    },
    WDLScore,
};

const INPUT_SIZE: usize = ThreatsExtended::INPUT_SIZE;
const HL_SIZE: usize = 8192;

const QA: i16 = 128;
const QB: i16 = 1024;

#[repr(C)]
#[repr(align(64))]
#[derive(Debug)]
pub struct ValueNetwork {
    l0: NetworkLayer<i8, INPUT_SIZE, HL_SIZE>,
    l1: TransposedNetworkLayer<i16, { HL_SIZE / 2 }, 16>,
    l2: NetworkLayer<f32, 16, 128>,
    l3: NetworkLayer<f32, 128, 3>,
}

impl ValueNetwork {
    pub fn forward(&self, board: &ChessBoard) -> WDLScore {
        let mut inputs: Accumulator<i16, HL_SIZE> = Accumulator::default();

        for (i, &bias) in inputs
            .values_mut()
            .iter_mut()
            .zip(self.l0.biases().values())
        {
            *i = i16::from(bias)
        }

        let mut features = [0u32; 512];
        let mut feature_count = 0usize;
        ThreatsExtended::map_inputs(board, |weight_idx| {
            features[feature_count] = weight_idx as u32;
            feature_count += 1;
        });

        let mut chunks8 = features[..feature_count].chunks_exact(8);
        for chunk in &mut chunks8 {
            let w0 = self.l0.weights()[chunk[0] as usize].values();
            let w1 = self.l0.weights()[chunk[1] as usize].values();
            let w2 = self.l0.weights()[chunk[2] as usize].values();
            let w3 = self.l0.weights()[chunk[3] as usize].values();
            let w4 = self.l0.weights()[chunk[4] as usize].values();
            let w5 = self.l0.weights()[chunk[5] as usize].values();
            let w6 = self.l0.weights()[chunk[6] as usize].values();
            let w7 = self.l0.weights()[chunk[7] as usize].values();
            let acc = inputs.values_mut();

            for i in 0..HL_SIZE {
                acc[i] += i16::from(w0[i])
                    + i16::from(w1[i])
                    + i16::from(w2[i])
                    + i16::from(w3[i])
                    + i16::from(w4[i])
                    + i16::from(w5[i])
                    + i16::from(w6[i])
                    + i16::from(w7[i]);
            }
        }

        let mut chunks = chunks8.remainder().chunks_exact(4);
        for chunk in &mut chunks {
            let w0 = self.l0.weights()[chunk[0] as usize].values();
            let w1 = self.l0.weights()[chunk[1] as usize].values();
            let w2 = self.l0.weights()[chunk[2] as usize].values();
            let w3 = self.l0.weights()[chunk[3] as usize].values();
            let acc = inputs.values_mut();

            for i in 0..HL_SIZE {
                acc[i] += i16::from(w0[i])
                    + i16::from(w1[i])
                    + i16::from(w2[i])
                    + i16::from(w3[i]);
            }
        }

        for &weight_idx in chunks.remainder() {
            let weights = self.l0.weights()[weight_idx as usize].values();
            let acc = inputs.values_mut();

            for i in 0..HL_SIZE {
                acc[i] += i16::from(weights[i]);
            }
        }

        let mut act = [0; HL_SIZE / 2];

        for (i, (&a, &b)) in act.iter_mut().zip(
            inputs
                .values()
                .iter()
                .take(HL_SIZE / 2)
                .zip(inputs.values().iter().skip(HL_SIZE / 2)),
        ) {
            let a = a.clamp(0, QA);
            let b = b.clamp(0, QA);
            *i = a * b;
        }

        let mut l1_neurons = [0; 16];

        for (neuron, weights) in l1_neurons.iter_mut().zip(self.l1.weights().iter()) {
            for (&value, &weight) in act.iter().zip(weights.values()) {
                *neuron += i32::from(value) * i32::from(weight);
            }
        }

        let mut l1_out: Accumulator<f32, 16> = Accumulator::default();

        for (out, (&value, &bias)) in l1_out
            .values_mut()
            .iter_mut()
            .zip(l1_neurons.iter().zip(self.l1.biases().values()))
        {
            *out = (value as f32 / f32::from(QA * QA) + f32::from(bias)) / f32::from(QB);
        }

        let mut l2_out = *self.l2.biases();
        for (value, weights) in l1_out.values().iter().zip(self.l2.weights()) {
            l2_out.madd(value.clamp(0.0, 1.0).powi(2), weights);
        }

        let mut out = *self.l3.biases();
        for (value, weights) in l2_out.values().iter().zip(self.l3.weights()) {
            out.madd(value.clamp(0.0, 1.0).powi(2), weights);
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
