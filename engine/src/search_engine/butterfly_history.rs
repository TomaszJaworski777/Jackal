use std::sync::atomic::{AtomicI16, Ordering};

use chess::{Move, Side, Square};

use crate::{search_engine::engine_options::EngineOptions, WDLScore};

#[derive(Debug)]
pub struct ButterflyHistory(Vec<AtomicI16>);

impl Clone for ButterflyHistory {
    fn clone(&self) -> Self {
        Self(self.0.iter().map(|x| AtomicI16::new(x.load(Ordering::Relaxed))).collect())
    }
}

impl ButterflyHistory {
    pub fn new() -> Self {
        Self((0..8192).map(|_| AtomicI16::new(0)).collect())
    }

    pub fn clear(&self) {
        for entry in &self.0 {
            entry.store(0, Ordering::Relaxed);
        }
    }

    pub fn get_bonus(&self, side: Side, mv: Move, options: &EngineOptions) -> f64 {
        f64::from(self.entry(side, mv).load(Ordering::Relaxed)) / (options.butterfly_bonus_scale() as f64)
    }

    pub fn update_entry(&self, side: Side, mv: Move, score: WDLScore, options: &EngineOptions) {
        let entry = self.entry(side, mv);

        let mut current_entry = entry.load(Ordering::Relaxed);
        loop {
            let delta = scale_bonus(current_entry, score.cp(), options.butterfly_reduction_factor() as i32);
            let new = current_entry.saturating_add(delta);
            match entry.compare_exchange(current_entry, new, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => current_entry = actual,
            }
        }
    }

    fn index(side: Side, from: Square, to: Square) -> usize {
        usize::from(side) * 4096 + usize::from(from) * 64 + usize::from(to)
    }

    fn entry(&self, side: Side, mv: Move) -> &AtomicI16 {
        &self.0[Self::index(side, mv.from_square(), mv.to_square())]
    }
}

fn scale_bonus(score: i16, bonus: i32, reduction_factor: i32) -> i16 {
    let bonus = bonus.clamp(i16::MIN as i32, i16::MAX as i32);
    let reduction = i32::from(score) * bonus.abs() / reduction_factor;
    let adjusted = bonus - reduction;
    adjusted.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}