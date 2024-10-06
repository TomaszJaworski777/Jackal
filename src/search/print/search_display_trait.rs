use crate::{search::Score, GameState};
use spear::{ChessPosition, Move};

use crate::{
    options::EngineOptions,
    search::{search_limits::SearchLimits, SearchStats},
};

#[allow(unused)]
pub trait SearchDisplay {
    const REFRESH_RATE: f32;

    fn new(position: &ChessPosition, engine_options: &EngineOptions) -> Self;
    #[allow(clippy::too_many_arguments)]
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
