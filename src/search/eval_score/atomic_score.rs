use std::sync::atomic::{AtomicU32, Ordering};

use super::Score;

pub struct AtomicScore(AtomicU32);

impl Default for AtomicScore {
    fn default() -> Self {
        // (f64::from(value) * f64::from(u32::MAX)) as u32
        let value = f32_to_u32(Score::default().win_chance());
        Self(AtomicU32::new(value))
    }
}

impl Clone for AtomicScore {
    fn clone(&self) -> Self {
        Self(AtomicU32::new(self.0.load(Ordering::Relaxed)))
    }
}

impl AtomicScore {
    pub fn store(&self, score: Score) {
        self.0.store(f32_to_u32(score.win_chance()), Ordering::Relaxed)
    }

    pub fn load(&self) -> Score {
        Score::new(u32_to_f32(self.0.load(Ordering::Relaxed)), 0.0)
    }
}

fn f32_to_u32(x: f32) -> u32 {
    (f64::from(x) * f64::from(u32::MAX)) as u32
}

fn u32_to_f32(x: u32) -> f32 {
    x as f32 / u32::MAX as f32
}