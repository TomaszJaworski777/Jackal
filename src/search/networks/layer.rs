use super::Accumulator;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NetworkLayer<T: Copy, const INPUTS: usize, const OUTPUTS: usize> {
    weights: [Accumulator<T, OUTPUTS>; INPUTS],
    biases: Accumulator<T, OUTPUTS>,
}

impl<T: Copy + Default, const INPUTS: usize, const OUTPUTS: usize> Default for NetworkLayer<T, INPUTS, OUTPUTS> {
    fn default() -> Self {
        Self {
            weights: [Accumulator::default(); INPUTS],
            biases: Accumulator::default(),
        }
    }
}

impl<T: Copy, const INPUTS: usize, const OUTPUTS: usize> NetworkLayer<T, INPUTS, OUTPUTS> {
    #[inline]
    pub fn weights(&self) -> &[Accumulator<T, OUTPUTS>; INPUTS] {
        &self.weights
    }

    #[inline]
    pub fn biases(&self) -> &Accumulator<T, OUTPUTS> {
        &self.biases
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TransposedNetworkLayer<T: Copy, const INPUTS: usize, const OUTPUTS: usize> {
    weights: [Accumulator<T, INPUTS>; OUTPUTS],
    biases: Accumulator<T, OUTPUTS>,
}

impl<T: Copy + Default, const INPUTS: usize, const OUTPUTS: usize> Default for TransposedNetworkLayer<T, INPUTS, OUTPUTS> {
    fn default() -> Self {
        Self {
            weights: [Accumulator::default(); OUTPUTS],
            biases: Accumulator::default(),
        }
    }
}

impl<T: Copy, const INPUTS: usize, const OUTPUTS: usize> TransposedNetworkLayer<T, INPUTS, OUTPUTS> {
    #[inline]
    pub fn weights(&self) -> &[Accumulator<T, INPUTS>; OUTPUTS] {
        &self.weights
    }

    #[inline]
    pub fn biases(&self) -> &Accumulator<T, OUTPUTS> {
        &self.biases
    }
}
