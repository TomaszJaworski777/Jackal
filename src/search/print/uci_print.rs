use spear::Move;

use crate::{
    options::EngineOptions,
    search::{SearchHelpers, SearchLimits, SearchStats},
};

use super::SearchPrinter;

pub struct UciPrint;
#[allow(unused)]
impl SearchPrinter for UciPrint {
    fn print_search_raport(
        search_stats: &SearchStats,
        engine_options: &EngineOptions,
        search_limits: &SearchLimits,
        score: f64,
    ) {
        println!(
            "info depth {} seldepth {} score {} time {} nodes {} nps {}",
            search_stats.avg_depth(),
            search_stats.max_depth(),
            SearchHelpers::score_into_cp(score as f32),
            search_stats.time_passed() as u128,
            search_stats.iters() as u128,
            search_stats.iters() as u128 * 1000 / search_stats.time_passed().max(1) as u128
        )
    }
    fn print_search_result(mv: Move, score: f64) {
        println!("bestmove {}", mv)
    }
}
