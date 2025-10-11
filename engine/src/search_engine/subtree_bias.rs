use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use chess::{ChessPosition, Piece};

use crate::{search_engine::engine_options::EngineOptions, WDLScore};

const BUCKET_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug)]
struct BiasEntry{
    weight: AtomicU32,
    error: AtomicU32,
    key: AtomicU64,
}

impl BiasEntry {
    fn new() -> Self {
        Self {
            weight: AtomicU32::new(0.0_f32.to_bits()),
            error: AtomicU32::new(0.0_f32.to_bits()),
            key: AtomicU64::new(0)
        }
    }

    fn key(&self) -> u64 { 
        self.key.load(Ordering::Relaxed)
    }

    fn error(&self, key: u64) -> f32 {
        if self.key() != key { 
            return 0.0;
        }

        let error = f32::from_bits(self.error.load(Ordering::Relaxed));
        let weight = f32::from_bits(self.weight.load(Ordering::Relaxed));

        error / weight
    }

    fn set(&self, error: f32, weight: f32, key: u64) {
        self.weight.store(weight.to_bits(), Ordering::Relaxed);
        self.error.store(error.to_bits(), Ordering::Relaxed);
        self.key.store(key, Ordering::Relaxed);
    }

    fn update(&self, error: f32, weight: f32, key: u64, options: &EngineOptions) {
        if self.key() != key {
            if weight < f32::from_bits(self.weight.load(Ordering::Relaxed)) * options.bias_replace_factor() as f32 {
                return;
            }

            self.set(error * options.bias_replace_boost() as f32, weight * options.bias_replace_boost() as f32, key);
            return;
        }

        atomic_add_f32(&self.error, error);
        atomic_add_f32(&self.weight, weight);
    }
}

fn atomic_add_f32(atomic: &AtomicU32, value: f32) {
    let mut old = atomic.load(Ordering::Relaxed);
    loop {
        let current = f32::from_bits(old);
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
                weight: AtomicU32::new(x.weight.load(Ordering::Relaxed)), 
                error: AtomicU32::new(x.error.load(Ordering::Relaxed)),
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

    fn update(&self, error: f32, weight: f32, key: u64, options: &EngineOptions) {
        let idx = self.index(key);
        self.0[idx].update(error, weight, key, options);
    }

    fn error(&self, key: u64) -> f32 {
        let idx = self.index(key);
        self.0[idx].error(key)
    }
}

#[derive(Debug, Clone)]
pub struct SubtreeBias {
    pawn_bucket: [BiasBucket; 2],
    minor_bucket: [BiasBucket; 2],
    major_bucket: [BiasBucket; 2]
}

impl SubtreeBias {
    pub fn new() -> Self {
        Self { 
            pawn_bucket: [BiasBucket::new(BUCKET_BYTES), BiasBucket::new(BUCKET_BYTES)],
            minor_bucket: [BiasBucket::new(BUCKET_BYTES), BiasBucket::new(BUCKET_BYTES)],
            major_bucket: [BiasBucket::new(BUCKET_BYTES), BiasBucket::new(BUCKET_BYTES)],
        }
    }

    pub fn update(&self, tree_score: f64, base_score: f64, visits: u32, position: &ChessPosition, options: &EngineOptions) {
        let weight = (visits.max(1) as f32).powf(options.bias_error_alpha() as f32);
        let error = base_score as f32 - tree_score as f32;

        let side = position.board().side();

        let key = pawn_key(position);
        self.pawn_bucket[usize::from(side)].update(error * weight, weight, key, options);

        let key = minor_key(position);
        self.minor_bucket[usize::from(side)].update(error * weight, weight, key, options);

        let key = major_key(position);
        self.major_bucket[usize::from(side)].update(error * weight, weight, key, options);
    }

    pub fn apply_bias(&self, score: &mut WDLScore, position: &ChessPosition, options: &EngineOptions) {
        let mut avg_error = 0.0;

        let side = position.board().side();

        let key = pawn_key(position);
        avg_error += self.pawn_bucket[usize::from(side)].error(key);

        let key = minor_key(position);
        avg_error += self.minor_bucket[usize::from(side)].error(key);

        let key = major_key(position);
        avg_error += self.major_bucket[usize::from(side)].error(key);

        let avg_error = (avg_error as f64) / 3.0;

        let biased_scalar = (score.single() - options.bias_lambda() * avg_error).clamp(0.0, 1.0);

        let new_w = (biased_scalar - 0.5 * score.draw_chance()).clamp(0.0, (1.0 - score.draw_chance()).max(0.0));
        *score = WDLScore::new(new_w, score.draw_chance());
    }
}

fn pawn_key(position: &ChessPosition) -> u64 {
    let side = position.board().side();
    let pawns = u64::from(position.board().piece_mask_for_side(Piece::PAWN, side));
    let king = u64::from(position.board().piece_mask_for_side(Piece::KING, side));
    pawns | king
}

fn minor_key(position: &ChessPosition) -> u64 {
    let side = position.board().side();
    let bishops = u64::from(position.board().piece_mask_for_side(Piece::BISHOP, side));
    let knights = u64::from(position.board().piece_mask_for_side(Piece::KNIGHT, side));
    bishops | knights
}

fn major_key(position: &ChessPosition) -> u64 {
    let side = position.board().side();
    let rooks = u64::from(position.board().piece_mask_for_side(Piece::ROOK, side));
    let queens = u64::from(position.board().piece_mask_for_side(Piece::QUEEN, side));
    let king = u64::from(position.board().piece_mask_for_side(Piece::KING, side));

    rooks | queens | king
}