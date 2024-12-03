use super::Accumulator;

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

impl<const INPUTS: usize, const OUTPUTS: usize> NetworkLayer<INPUTS, OUTPUTS> {
    #[inline]
    pub fn weights(&self) -> &[Accumulator<OUTPUTS>; INPUTS] {
        &self.weights
    }

    #[inline]
    pub fn biases(&self) -> &Accumulator<OUTPUTS> {
        &self.biases
    }

    pub fn forward(&self, inputs: &Accumulator<INPUTS>) -> Accumulator<OUTPUTS> {
        let mut result = self.biases;

        for (neuron, weights) in inputs.values().iter().zip(self.weights.iter()) {
            let activated = neuron.clamp(0.0, 1.0).powi(2);
            result.madd(activated, weights);
        }

        result
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
