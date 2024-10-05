use crate::{search::Score, GameState};
use spear::Move;

use crate::{
    options::EngineOptions,
    search::{search_limits::SearchLimits, SearchStats},
};

#[allow(unused)]
pub trait SearchDisplay {
    fn new() -> Self;
    fn print_search_start(
        &mut self,
        search_stats: &SearchStats,
        engine_options: &EngineOptions,
        search_limits: &SearchLimits,
    ) {
    }
    fn print_search_raport<const FINAL: bool>(
        &mut self,
        search_stats: &SearchStats,
        engine_options: &EngineOptions,
        search_limits: &SearchLimits,
        usage: f32,
        score: Score,
        state: GameState,
        pv: &[Move],
    ) {
    }
    fn print_search_result(&self, mv: Move, score: Score) {}
}
