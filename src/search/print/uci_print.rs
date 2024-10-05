use crate::{search::Score, GameState};
use spear::{ChessPosition, Move};

use crate::{
    options::EngineOptions,
    search::{SearchLimits, SearchStats},
};

use super::SearchDisplay;

pub struct UciPrint;
#[allow(unused)]
impl SearchDisplay for UciPrint {
    fn new(position: &ChessPosition, engine_options: &EngineOptions) -> Self {
        UciPrint
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
        let mut pv_string = String::new();
        for mv in pv {
            pv_string.push_str(format!("{} ", mv).as_str())
        }

        let mut score_text = match state {
            GameState::Drawn => "score cp 0".to_string(),
            GameState::Won(x) => format!("score mate {}", (x as f32 / 2.0).ceil() as u32),
            GameState::Lost(x) => format!("score mate -{}", (x as f32 / 2.0).ceil() as u32),
            _ => format!("score cp {}", score.as_cp()),
        };

        if engine_options.show_wdl() {
            score_text.push_str(" wdl 534 321 123");
        }

        println!(
            "info depth {} seldepth {} {} time {} nodes {} nps {} hashfull {} pv {}",
            search_stats.avg_depth(),
            search_stats.max_depth(),
            score_text,
            search_stats.time_passed() as u128,
            search_stats.iters() as u128,
            search_stats.iters() as u128 * 1000 / search_stats.time_passed().max(1) as u128,
            (usage * 1000.0) as u32,
            pv_string
        )
    }
    fn print_search_result(&self, mv: Move, score: Score) {
        println!("bestmove {}", mv)
    }
}
