use std::sync::atomic::{AtomicU32, Ordering};

use super::Score;

pub struct AtomicScore(AtomicU32);

impl Default for AtomicScore {
    fn default() -> Self {
        Self(AtomicU32::new(u32::from(Score::default())))
    }
}

impl Clone for AtomicScore {
    fn clone(&self) -> Self {
        Self(AtomicU32::new(self.0.load(Ordering::Relaxed)))
    }
}

impl AtomicScore {
    pub fn store(&self, score: Score) {
        self.0.store(u32::from(score), Ordering::Relaxed)
    }

    pub fn load(&self) -> Score {
        Score::from(self.0.load(Ordering::Relaxed))
    }
}
