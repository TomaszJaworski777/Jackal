use std::sync::atomic::{AtomicU64, Ordering};

use chess::{ChessPosition, Piece};

use crate::{search_engine::engine_options::EngineOptions, WDLScore};

const BUCKET_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug)]
struct BiasEntry{
    weight: AtomicU64,
    error: AtomicU64,
    key: AtomicU64,
}

impl BiasEntry {
    fn new() -> Self {
        Self {
            weight: AtomicU64::new(0.0_f64.to_bits()),
            error: AtomicU64::new(0.0_f64.to_bits()),
            key: AtomicU64::new(0)
        }
    }

    fn key(&self) -> u64 { 
        self.key.load(Ordering::Relaxed)
    }

    fn error(&self, key: u64) -> f64 {
        if self.key() != key { 
            return 0.0;
        }

        let error = f64::from_bits(self.error.load(Ordering::Relaxed));
        let weight = f64::from_bits(self.weight.load(Ordering::Relaxed));

        error / weight
    }

    fn set(&self, error: f64, weight: f64, key: u64) {
        self.weight.store(weight.to_bits(), Ordering::Relaxed);
        self.error.store(error.to_bits(), Ordering::Relaxed);
        self.key.store(key, Ordering::Relaxed);
    }

    fn update(&self, error: f64, weight: f64, key: u64, options: &EngineOptions) {
        if self.key() != key {
            if weight < f64::from_bits(self.weight.load(Ordering::Relaxed)) * options.bias_replace_factor() {
                return;
            }

            self.set(error, weight, key);
        }

        atomic_add_f64(&self.error, error);
        atomic_add_f64(&self.weight, weight);
    }
}

fn atomic_add_f64(atomic: &AtomicU64, value: f64) {
    let mut old = atomic.load(Ordering::Relaxed);
    loop {
        let current = f64::from_bits(old);
        let new = (current + value).to_bits();
        match atomic.compare_exchange_weak(old, new, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(now) => old = now,
        }
    }
}

#[derive(Debug)]
struct BiasBucket(Vec<BiasEntry>);

impl Clone for BiasBucket {
    fn clone(&self) -> Self {
        Self(self.0.iter().map(|x| {
            BiasEntry { 
                weight: AtomicU64::new(x.weight.load(Ordering::Relaxed)), 
                error: AtomicU64::new(x.error.load(Ordering::Relaxed)),
                key: AtomicU64::new(x.key.load(Ordering::Relaxed)),
            }
        }).collect())
    }
}

impl BiasBucket {
    fn new(bytes: usize) -> Self {
        let size = bytes / std::mem::size_of::<BiasEntry>();
        Self((0..size).map(|_| BiasEntry::new()).collect())
    }

    fn index(&self, key: u64) -> usize {
        (key % (self.0.len() as u64)) as usize
    }

    fn update(&self, error: f64, weight: f64, key: u64, options: &EngineOptions) {
        let idx = self.index(key);
        self.0[idx].update(error, weight, key, options);
    }

    fn error(&self, key: u64) -> f64 {
        let idx = self.index(key);
        self.0[idx].error(key)
    }
}

#[derive(Debug, Clone)]
pub struct SubtreeBias {
    pawn_bucket: [BiasBucket; 2],
    bishop_bucket: [BiasBucket; 2]
}

impl SubtreeBias {
    pub fn new() -> Self {
        Self { 
            pawn_bucket: [BiasBucket::new(BUCKET_BYTES), BiasBucket::new(BUCKET_BYTES)],
            bishop_bucket: [BiasBucket::new(BUCKET_BYTES), BiasBucket::new(BUCKET_BYTES)]
        }
    }

    pub fn update(&self, tree_score: f64, base_score: f64, visits: u32, position: &ChessPosition, options: &EngineOptions) {
        let weight = (visits.max(1) as f64).powf(options.bias_error_alpha());
        let error = base_score - tree_score;

        let side = position.board().side();

        let key = u64::from(position.board().piece_mask_for_side(Piece::PAWN, side));
        self.pawn_bucket[usize::from(side)].update(error * weight, weight, key, options);

        let key = u64::from(position.board().piece_mask_for_side(Piece::BISHOP, side));
        self.bishop_bucket[usize::from(side)].update(error * weight, weight, key, options);
    }

    pub fn apply_bias(&self, score: &mut WDLScore, position: &ChessPosition, options: &EngineOptions) {
        let mut avg_error = 0.0;

        let side = position.board().side();

        let key = u64::from(position.board().piece_mask_for_side(Piece::PAWN, side));
        avg_error += self.pawn_bucket[usize::from(side)].error(key);

        let key = u64::from(position.board().piece_mask_for_side(Piece::BISHOP, side));
        avg_error += self.bishop_bucket[usize::from(side)].error(key);

        let avg_error = (avg_error as f64) / 2.0;

        let biased_scalar = (score.single() - options.bias_lambda() * avg_error).clamp(0.0, 1.0);

        let new_w = (biased_scalar - 0.5 * score.draw_chance()).clamp(0.0, (1.0 - score.draw_chance()).max(0.0));
        *score = WDLScore::new(new_w, score.draw_chance());
    }
}