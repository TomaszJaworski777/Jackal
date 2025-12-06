use std::sync::atomic::{AtomicI16, Ordering};

use chess::{Move, Piece, Side};

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
        Self((0..49152).map(|_| AtomicI16::new(0)).collect())
    }

    pub fn clear(&self) {
        for entry in &self.0 {
            entry.store(0, Ordering::Relaxed);
        }
    }

    pub fn get_bonus(&self, piece: Piece, side: Side, mv: Move, options: &EngineOptions) -> f64 {
        f64::from(self.entry(piece, side, mv).load(Ordering::Relaxed)) / 
        match piece {
            Piece::PAWN => options.butterfly_pawn_scale(),
            Piece::KNIGHT => options.butterfly_knight_scale(),
            Piece::BISHOP => options.butterfly_bishop_scale(),
            Piece::ROOK => options.butterfly_rook_scale(),
            Piece::QUEEN => options.butterfly_queen_scale(),
            Piece::KING => options.butterfly_king_scale(),
            _ => unreachable!()
        } 
    }

    pub fn update_entry(&self, piece: Piece, side: Side, mv: Move, score: WDLScore, options: &EngineOptions) {
        let score = (-400.0 * ((1.0 / score.single().clamp(0.001, 0.999)) - 1.0).ln()).round() as i32;
        let entry = self.entry(piece, side, mv);

        let mut current_entry = entry.load(Ordering::Relaxed);
        loop {
            let delta = scale_bonus(current_entry, score, match piece {
                Piece::PAWN => options.butterfly_pawn_reduction(),
                Piece::KNIGHT => options.butterfly_knight_reduction(),
                Piece::BISHOP => options.butterfly_bishop_reduction(),
                Piece::ROOK => options.butterfly_rook_reduction(),
                Piece::QUEEN => options.butterfly_queen_reduction(),
                Piece::KING => options.butterfly_king_reduction(),
                _ => unreachable!()
            } as i32);
            let new = current_entry.saturating_add(delta);
            match entry.compare_exchange(current_entry, new, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => current_entry = actual,
            }
        }
    }

    fn entry(&self, piece: Piece, side: Side, mv: Move) -> &AtomicI16 {
        &self.0[usize::from(piece) * 8192 + usize::from(side) * 4096 + usize::from(mv.from_square()) * 64 + usize::from(mv.to_square())]
    }
}

fn scale_bonus(score: i16, bonus: i32, reduction_factor: i32) -> i16 {
    let bonus = bonus.clamp(i16::MIN as i32, i16::MAX as i32);
    let reduction = i32::from(score) * bonus.abs() / reduction_factor;
    let adjusted = bonus - reduction;
    adjusted.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}