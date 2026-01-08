use std::{ops::Mul, sync::atomic::{AtomicU64, Ordering}};

use chess::{ChessBoard, Piece};

use crate::search_engine::engine_params::EngineParams;

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

    pub fn cp(&self) -> i32 {
        let w = self.win_chance();
        let l = self.lose_chance();
        let wl = w - l;

        let tan_cp = 105.20 * (1.342 * wl).tan() + 32.94 * (1.342 * wl).tan().powi(3);
        
        if w.min(l) > 0.002 {
            let a = (1.0 / l.clamp(0.0001, 0.9999) - 1.0).ln();
            let b = (1.0 / w.clamp(0.0001, 0.9999) - 1.0).ln();
            let denom = a + b;

            if denom.abs() > 0.01 {
                let mu = (a - b) / denom;
                let mu_cp = mu * 100.0;

                if mu_cp.abs() > tan_cp.abs() || mu_cp.abs() < 100.0 { 
                    return mu_cp as i32;
                }
            }
        };

        tan_cp.clamp(-30000.0, 30000.0) as i32
    }

    pub fn apply_50mr_and_draw_scaling(&mut self, half_move: u8, depth: f64, options: &EngineParams) {
        let s = (0.01 * half_move as f64).powf(options.power_50mr()).min(options.cap_50mr()) + (depth.powf(options.depth_scaling_power()) * options.depth_scaling()).min(options.depth_scaling_cap());
        let win_delta = self.win_chance() * s; 
        let lose_delta = self.lose_chance() * s; 

        self.0 -= win_delta;
        self.1 += win_delta + lose_delta;
    }

    pub fn apply_sharpness_scaling(&mut self, params: &EngineParams) {
        let draw_chance = self.draw_chance();

        let scale = draw_chance * params.sharpness_scale()
            + draw_chance * draw_chance * params.sharpness_scale_2();

        let scale = (1.0 - scale).max(0.0);
        let score = self.single();
        let scale = 1.0 / (1.0 + (-scale * (score / (1.0 - score)).ln()).exp());

        let q = (self.win_chance() + self.lose_chance()).clamp(0.0001, 1.0);
        let p = self.win_chance() / q;

        let new_q = if (p - 0.5).abs() < 1e-9 {
            0.0
        } else {
            ((scale - 0.5) / (p - 0.5)).clamp(0.0, 1.0)
        };

        self.0 = p * new_q;
        self.1 = 1.0 - new_q;
    }

    pub fn apply_contempt(&mut self, contempt: i64) {
        const EPS: f64 = 0.0001;

        if contempt == 0 {
            return;
        }

        let (w, l) = (self.win_chance(), self.lose_chance());

        if w < EPS || l < EPS || w > 1.0 - EPS || l > 1.0 - EPS {
            return;
        }

        let a = (1.0 / l - 1.0).ln();
        let b = (1.0 / w - 1.0).ln();
        let denom = a + b;

        if denom.abs() < 0.000001 {
            return;
        }

        let uncertainty = 2.0 / denom;
        let advantage = (a - b) / denom;

        let shift = (uncertainty.powi(2) * contempt as f64 * std::f64::consts::LN_10 / (400.0 * 16.0)).clamp(-0.8, 0.8);

        let new_advantage = advantage + shift;

        let new_w = fast_logistic((-1.0 + new_advantage) / uncertainty);
        let new_l = fast_logistic((-1.0 - new_advantage) / uncertainty);
        let new_d = (1.0 - new_w - new_l).clamp(0.0, 1.0);

        self.0 = new_w;
        self.1 = new_d;
    }
}

impl Mul<u32> for WDLScore {
    type Output = WDLScore;

    fn mul(self, rhs: u32) -> Self::Output {
        Self(self.win_chance() * f64::from(rhs), self.draw_chance() * f64::from(rhs))
    }
}

fn fast_logistic(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}