use std::sync::atomic::{AtomicU32, Ordering};

use chess::{ChessPosition, Piece};

use crate::{search_engine::engine_options::EngineOptions, WDLScore};

const BUCKET_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug)]
struct BiasEntry{
    //score: AtomicU32,
    error: AtomicU32
}

impl BiasEntry {
    fn new() -> Self {
        Self {
            //score: AtomicU32::new(0.0_f32.to_bits()),
            error: AtomicU32::new(0.0_f32.to_bits()),
        }
    }

    fn error(&self) -> f32 {
        f32::from_bits(self.error.load(Ordering::Relaxed))
    }

    fn set(&self, _score: f32, error: f32) {
        //self.score.store(score.to_bits(), Ordering::Relaxed);
        self.error.store(error.to_bits(), Ordering::Relaxed);
    }

    fn update(&self, score: f32, error: f32) {
        self.set(score, error);
    }
}

#[derive(Debug)]
struct BiasBucket(Vec<BiasEntry>);

impl Clone for BiasBucket {
    fn clone(&self) -> Self {
        Self(self.0.iter().map(|x| {
            let result = BiasEntry::new();
            result.set(0.0, x.error());
            result
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

    fn update(&self, score: f32, error: f32, key: u64) {
        let idx = self.index(key);
        self.0[idx].update(score, error);
    }

    fn error(&self, key: u64) -> f32 {
        let idx = self.index(key);
        self.0[idx].error()
    }
}

#[derive(Debug, Clone)]
pub struct SubtreeBias {
    pawn_bucket: [BiasBucket; 2]
}

impl SubtreeBias {
    pub fn new() -> Self {
        Self { 
            pawn_bucket: [BiasBucket::new(BUCKET_BYTES), BiasBucket::new(BUCKET_BYTES)]
        }
    }

    pub fn update(&self, tree_score: f32, base_score: f32, position: &ChessPosition) {
        let error = base_score - tree_score;

        let side = position.board().side();

        let pawn_key = u64::from(position.board().piece_mask_for_side(Piece::PAWN, side));
        self.pawn_bucket[usize::from(side)].update(tree_score, error, pawn_key);
    }

    pub fn apply_bias(&self, score: &mut WDLScore, position: &ChessPosition, options: &EngineOptions) {
        let mut avg_error = 0.0;

        let side = position.board().side();

        let pawn_key = u64::from(position.board().piece_mask_for_side(Piece::PAWN, side));
        avg_error += self.pawn_bucket[usize::from(side)].error(pawn_key);

        let avg_error = (avg_error as f64) / 1.0;

        let biased_scalar = (score.single() - options.bias_lambda() * avg_error).clamp(0.0, 1.0);

        let new_w = (biased_scalar - 0.5 * score.draw_chance()).clamp(0.0, 1.0 - score.draw_chance());
        *score = WDLScore::new(new_w, score.draw_chance());
    }
}