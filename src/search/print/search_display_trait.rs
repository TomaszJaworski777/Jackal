use crate::{search::Score, GameState};
use spear::Move;

use crate::{
    options::EngineOptions,
    search::{search_limits::SearchLimits, SearchStats},
};

#[allow(unused)]
pub trait SearchDisplay {
    fn print_search_start(
        search_stats: &SearchStats,
        engine_options: &EngineOptions,
        search_limits: &SearchLimits,
    ) {
    }
    fn print_search_raport(
        search_stats: &SearchStats,
        engine_options: &EngineOptions,
        search_limits: &SearchLimits,
        score: Score,
        state: GameState,
        pv: &[Move],
    ) {
    }
    fn print_search_result(mv: Move, score: Score) {}
}
