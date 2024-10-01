use crate::GameState;
use spear::Move;

use crate::{
    options::EngineOptions,
    search::{SearchHelpers, SearchLimits, SearchStats},
};

use super::SearchDisplay;

pub struct UciPrint;
#[allow(unused)]
impl SearchDisplay for UciPrint {
    fn print_search_raport(
        search_stats: &SearchStats,
        engine_options: &EngineOptions,
        search_limits: &SearchLimits,
        score: f64,
        state: GameState,
        pv: &[Move],
    ) {
        let mut pv_string = String::new();
        for mv in pv {
            pv_string.push_str(format!("{} ", mv).as_str())
        }

        let score_text = match state {
            GameState::Drawn => "score cp 0".to_string(),
            GameState::Won(x) => format!("score mate {}", (x as f32 / 2.0).ceil() as u32),
            GameState::Lost(x) => format!("score mate -{}", (x as f32 / 2.0).ceil() as u32),
            _ => format!("score cp {}", SearchHelpers::score_into_cp(score as f32)),
        };

        println!(
            "info depth {} seldepth {} {} time {} nodes {} nps {} pv {}",
            search_stats.avg_depth(),
            search_stats.max_depth(),
            score_text,
            search_stats.time_passed() as u128,
            search_stats.iters() as u128,
            search_stats.iters() as u128 * 1000 / search_stats.time_passed().max(1) as u128,
            pv_string
        )
    }
    fn print_search_result(mv: Move, score: f64) {
        println!("bestmove {}", mv)
    }
}
