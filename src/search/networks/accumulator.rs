#[derive(Clone, Copy)]
#[repr(C)]
pub struct Accumulator<const HIDDEN: usize> {
    pub vals: [f32; HIDDEN],
}

impl<const SIZE: usize> Default for Accumulator<SIZE> {
    fn default() -> Self {
        Self { vals: [0.0; SIZE] }
    }
}

impl<const HIDDEN: usize> Accumulator<HIDDEN> {
    pub fn madd(&mut self, mul: f32, other: &Self) {
        for (i, &j) in self.vals.iter_mut().zip(other.vals.iter()) {
            *i += mul * j;
        }
    }

    #[inline]
    pub fn values(&self) -> &[f32; HIDDEN] {
        &self.vals
    }

    #[inline]
    pub fn values_mut(&mut self) -> &mut [f32; HIDDEN] {
        &mut self.vals
    }
}
