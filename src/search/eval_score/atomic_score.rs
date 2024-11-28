use std::sync::atomic::{AtomicU32, Ordering};

use super::Score;

pub struct AtomicScore(AtomicU32, AtomicU32);

impl Default for AtomicScore {
    fn default() -> Self {
        let w = f32_to_u32(Score::default().win_chance());
        let d = f32_to_u32(Score::default().draw_chance());
        Self(AtomicU32::new(w), AtomicU32::new(d))
    }
}

impl Clone for AtomicScore {
    fn clone(&self) -> Self {
        Self(AtomicU32::new(self.0.load(Ordering::Relaxed)),
            AtomicU32::new(self.1.load(Ordering::Relaxed)))
    }
}

impl AtomicScore {
    pub fn store(&self, score: Score) {
        self.0.store(f32_to_u32(score.win_chance()), Ordering::Relaxed);
        self.1.store(f32_to_u32(score.draw_chance()), Ordering::Relaxed);
    }

    pub fn load(&self) -> Score {
        let w = u32_to_f32(self.0.load(Ordering::Relaxed));
        let d = u32_to_f32(self.1.load(Ordering::Relaxed));
        Score::new(w, d)
    }
}

fn f32_to_u32(x: f32) -> u32 {
    (f64::from(x) * f64::from(u32::MAX)) as u32
}

fn u32_to_f32(x: u32) -> f32 {
    x as f32 / u32::MAX as f32
}