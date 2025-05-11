use std::ops::{AddAssign, Mul};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Accumulator<T: Copy, const HIDDEN: usize> {
    pub vals: [T; HIDDEN],
}

impl<T: Copy + Default, const SIZE: usize> Default for Accumulator<T, SIZE> {
    fn default() -> Self {
        Self { vals: [T::default(); SIZE] }
    }
}

impl<T: AddAssign<T> + Copy + Mul<T, Output = T>, const HIDDEN: usize> Accumulator<T, HIDDEN> {
    pub fn madd(&mut self, mul: T, other: &Self) {
        for (i, &j) in self.vals.iter_mut().zip(other.vals.iter()) {
            *i += mul * j;
        }
    }

    #[inline]
    pub fn values(&self) -> &[T; HIDDEN] {
        &self.vals
    }

    #[inline]
    pub fn values_mut(&mut self) -> &mut [T; HIDDEN] {
        &mut self.vals
    }
}
