use std::ops::{AddAssign, Mul};

use super::{accumulator::QuantizedAccumulator, Accumulator};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NetworkLayer<const INPUTS: usize, const OUTPUTS: usize> {
    weights: [Accumulator<OUTPUTS>; INPUTS],
    biases: Accumulator<OUTPUTS>,
}

impl<const INPUTS: usize, const OUTPUTS: usize> Default for NetworkLayer<INPUTS, OUTPUTS> {
    fn default() -> Self {
        Self {
            weights: [Accumulator::default(); INPUTS],
            biases: Accumulator::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QunatisedNetworkLayer<T: Copy, const INPUTS: usize, const OUTPUTS: usize> {
    pub weights: [QuantizedAccumulator<T, OUTPUTS>; INPUTS],
    pub biases: QuantizedAccumulator<T, OUTPUTS>,
}

impl<T: Copy + Default, const INPUTS: usize, const OUTPUTS: usize> Default
    for QunatisedNetworkLayer<T, INPUTS, OUTPUTS>
{
    fn default() -> Self {
        Self {
            weights: [QuantizedAccumulator::default(); INPUTS],
            biases: QuantizedAccumulator::default(),
        }
    }
}

impl<const INPUTS: usize, const OUTPUTS: usize> NetworkLayer<INPUTS, OUTPUTS> {
    #[inline]
    pub fn weights(&self) -> &[Accumulator<OUTPUTS>; INPUTS] {
        &self.weights
    }

    #[inline]
    pub fn biases(&self) -> &Accumulator<OUTPUTS> {
        &self.biases
    }

    pub fn forward_relu(&self, inputs: &Accumulator<INPUTS>) -> Accumulator<OUTPUTS> {
        let mut result = self.biases;

        for (neuron, weights) in inputs.values().iter().zip(self.weights.iter()) {
            let activated = neuron.max(0.0);
            result.madd(activated, weights);
        }

        result
    }
}

impl<
        T: Copy + AddAssign<T> + Mul<T, Output = T> + Default,
        const INPUTS: usize,
        const OUTPUTS: usize,
    > QunatisedNetworkLayer<T, INPUTS, OUTPUTS>
{
    #[inline]
    pub fn weights(&self) -> &[QuantizedAccumulator<T, OUTPUTS>; INPUTS] {
        &self.weights
    }

    #[inline]
    pub fn biases(&self) -> &QuantizedAccumulator<T, OUTPUTS> {
        &self.biases
    }
}
