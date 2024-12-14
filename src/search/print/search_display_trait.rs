use crate::{search::Score, GameState, Tree};
use spear::{ChessPosition, Move};

use crate::{
    options::EngineOptions,
    search::{search_limits::SearchLimits, SearchStats},
};

#[allow(unused)]
pub trait SearchDisplay: Send + Sync {
    const REFRESH_RATE: f32;

    fn new(position: &ChessPosition, engine_options: &EngineOptions, tree: &Tree) -> Self;
    #[allow(clippy::too_many_arguments)]
    fn print_search_raport<const FINAL: bool>(
        &mut self,
        search_stats: &SearchStats,
        engine_options: &EngineOptions,
        search_limits: &SearchLimits,
        usage: f32,
        pvs: &Vec<(Score, GameState, Vec<Move>)>,
    ) {
    }
    fn print_search_result(&self, mv: Move, score: Score) {}
}
