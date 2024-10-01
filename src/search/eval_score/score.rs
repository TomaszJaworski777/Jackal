#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Score(f32);

impl From<f32> for Score {
    fn from(value: f32) -> Self { 
        Self(value)
    }
}

impl From<f64> for Score {
    fn from(value: f64) -> Self { 
        Self(value as f32)
    }
}

impl From<Score> for f32 {
    fn from(value: Score) -> Self { 
        value.0
    }
}

impl From<Score> for f64 {
    fn from(value: Score) -> Self { 
        value.0 as f64
    }
}

impl From<u32> for Score {
    fn from(value: u32) -> Self { 
        let new_value = f64::from(value) / f64::from(u32::MAX);
        Self::from(new_value)
    }
}

impl From<Score> for u32 {
    fn from(value: Score) -> Self { 
        (f64::from(value) * f64::from(u32::MAX)) as u32
    }
}

impl Score {
    pub const WIN: Self = Self(1.0);
    pub const DRAW: Self = Self(0.5);
    pub const LOSE: Self = Self(0.0);

    pub fn single(&self) -> f32 {
        self.0
    }

    pub fn as_cp(&self) -> i32 {
        (-400.0 * (1.0 / self.single().clamp(0.0, 1.0) - 1.0).ln()) as i32
    }

    pub fn as_cp_f32(&self) -> f32 {
        self.as_cp() as f32 / 100.0
    }

    pub fn reversed(&self) -> Self {
        Self(1.0 - self.0)
    }
}