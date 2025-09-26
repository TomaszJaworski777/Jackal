use std::{ops::Mul, sync::atomic::{AtomicU64, Ordering}};

use crate::search_engine::engine_options::EngineOptions;

pub const SCORE_SCALE: u32 = 1024 * 64;

#[derive(Debug, Default)]
pub struct AtomicWDLScore(AtomicU64, AtomicU64);

impl Clone for AtomicWDLScore {
    fn clone(&self) -> Self {
        Self(
            AtomicU64::new(self.0.load(Ordering::Relaxed)), 
            AtomicU64::new(self.1.load(Ordering::Relaxed))
        )
    }
}

impl From<WDLScore> for AtomicWDLScore {
    fn from(value: WDLScore) -> Self {
        let win_chance = (value.win_chance() as f64 * f64::from(SCORE_SCALE)) as u64;
        let draw_chance = (value.draw_chance() as f64 * f64::from(SCORE_SCALE)) as u64;

        Self(
            AtomicU64::new(win_chance), 
            AtomicU64::new(draw_chance)
        )
    }
}

impl AtomicWDLScore {
    #[inline]
    pub fn get_score_with_visits(&self, visits: u32) -> WDLScore {
        let score = self.get_score();
        let win_chance = score.win_chance() / f64::from(visits.max(1));
        let draw_chance = score.draw_chance() / f64::from(visits.max(1));
        WDLScore(win_chance, draw_chance)
    }

    #[inline]
    pub fn get_score(&self) -> WDLScore {
        let win_chance = self.0.load(Ordering::Relaxed) as f64 / f64::from(SCORE_SCALE);
        let draw_chance = self.1.load(Ordering::Relaxed) as f64 / f64::from(SCORE_SCALE);
        WDLScore(win_chance, draw_chance)
    }

    #[inline]
    pub fn clear(&self) {
        self.0.store(0, Ordering::Relaxed);
        self.1.store(0, Ordering::Relaxed);
    }

    #[inline]
    pub fn store(&self, value: WDLScore) {
        let win_chance = (value.win_chance() as f64 * f64::from(SCORE_SCALE)) as u64;
        let draw_chance = (value.draw_chance() as f64 * f64::from(SCORE_SCALE)) as u64;

        self.0.store(win_chance, Ordering::Relaxed);
        self.1.store(draw_chance, Ordering::Relaxed);
    }

    #[inline]
    pub fn add(&self, rhs: WDLScore) {
        let win_chance = (rhs.win_chance() as f64 * f64::from(SCORE_SCALE)) as u64;
        let draw_chance = (rhs.draw_chance() as f64 * f64::from(SCORE_SCALE)) as u64;

        self.0.fetch_add(win_chance, Ordering::Relaxed);
        self.1.fetch_add(draw_chance, Ordering::Relaxed);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct WDLScore(f64, f64);
impl WDLScore {
    pub const WIN: Self = Self(1.0, 0.0);
    pub const DRAW: Self = Self(0.0, 1.0);
    pub const LOSE: Self = Self(0.0, 0.0);

    #[inline]
    pub const fn new(win_chance: f64, draw_chance: f64) -> Self {
        Self(win_chance, draw_chance)
    }

    #[inline]
    pub const fn win_chance(&self) -> f64 {
        self.0
    }

    #[inline]
    pub const fn draw_chance(&self) -> f64 {
        self.1
    }

    #[inline]
    pub const fn lose_chance(&self) -> f64 {
        1.0 - self.win_chance() - self.draw_chance()
    }

    #[inline]
    pub const fn reversed(&self) -> Self {
        Self(self.lose_chance(), self.draw_chance())
    }

    #[inline]
    pub const fn single(&self) -> f64 {
        self.win_chance() + self.draw_chance() * 0.5
    }

    #[inline]
    pub const fn single_with_score(&self, draw_score: f64) -> f64 {
        (self.win_chance() + self.draw_chance() * draw_score).clamp(0.0, 1.0)
    }

    #[inline]
    pub fn cp(&self) -> i32 {
        let score = (-246.631 * (1.0 / self.single() - 1.0).ln()) as i32;
        score.clamp(-30000, 30000)
    }

    #[inline]
    pub fn apply_50mr(&mut self, half_move: u8, depth: f64, options: &EngineOptions) {
        let s = (0.01 * half_move as f64).powf(options.draw_scaling_power()).min(options.draw_scaling_cap()) + depth * options.depth_scaling();
        let win_delta = self.win_chance() * s; 
        let lose_delta = self.lose_chance() * s; 

        self.0 -= win_delta;
        self.1 += win_delta + lose_delta;
    }
}

impl Mul<u32> for WDLScore {
    type Output = WDLScore;

    fn mul(self, rhs: u32) -> Self::Output {
        Self(self.win_chance() * f64::from(rhs), self.draw_chance() * f64::from(rhs))
    }
}