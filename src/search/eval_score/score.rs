#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Score(f32, f32);

impl Score {
    pub const WIN: Self = Self(1.0, 0.0);
    pub const DRAW: Self = Self(0.0, 1.0);
    pub const LOSE: Self = Self(0.0, 0.0);

    pub fn new(win_chance: f32, draw_chance: f32) -> Self {
        Self(win_chance, draw_chance)
    }

    pub fn single_us(&self) -> f32 {
        self.0 * (1.0 - self.1) + self.1 / 2.0
    }

    pub fn single_them(&self) -> f32 {
        self.0 + self.1 / 2.0
    }

    pub fn win_chance(&self) -> f32 {
        self.0
    }

    pub fn draw_chance(&self) -> f32 {
        self.1
    }

    pub fn lose_chance(&self) -> f32 {
        1.0 - self.0 - self.1
    }

    pub fn as_cp_us(&self) -> i32 {
        (-400.0 * (1.0 / self.single_us().clamp(0.0, 1.0) - 1.0).ln()) as i32
    }

    pub fn as_cp_f32_us(&self) -> f32 {
        self.as_cp_us() as f32 / 100.0
    }

    pub fn as_cp_them(&self) -> i32 {
        (-400.0 * (1.0 / self.single_them().clamp(0.0, 1.0) - 1.0).ln()) as i32
    }

    pub fn as_cp_f32_them(&self) -> f32 {
        self.as_cp_them() as f32 / 100.0
    }

    pub fn reversed(&self) -> Self {
        Self(self.lose_chance(), self.1)
    }
}
