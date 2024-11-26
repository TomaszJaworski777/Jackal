use crate::{search::Score, GameState, Tree};
use spear::{ChessPosition, Move};

use crate::{
    options::EngineOptions,
    search::{SearchLimits, SearchStats},
};

use super::SearchDisplay;

pub struct UciPrint;
#[allow(unused)]
impl SearchDisplay for UciPrint {
    const REFRESH_RATE: f32 = 1.0;

    fn new(position: &ChessPosition, engine_options: &EngineOptions, tree: &Tree) -> Self {
        Self
    }

    fn print_search_raport<const FINAL: bool>(
        &mut self,
        search_stats: &SearchStats,
        engine_options: &EngineOptions,
        search_limits: &SearchLimits,
        usage: f32,
        pvs: &Vec<(Score, GameState, Vec<Move>)>
    ) {
        for multi_pv_idx in 0..pvs.len() {
            let (score, state, pv) = &pvs[multi_pv_idx];

            if pv.is_empty() {
                return;
            }

            let mut pv_string = String::new();
            for mv in pv {
                pv_string.push_str(format!("{} ", mv).as_str())
            }

            let mut score_text = match *state {
                GameState::Drawn => "score cp 0".to_string(),
                GameState::Won(x) => format!("score mate {}", ((x+1) as f32 / 2.0).ceil() as u32),
                GameState::Lost(x) => format!("score mate -{}", ((x+1) as f32 / 2.0).ceil() as u32),
                _ => format!("score cp {}", score.as_cp()),
            };
    
            if engine_options.show_wdl() {
                score_text.push_str(&format!(
                    " wdl {} {} {}",
                    (score.win_chance() * 1000.0) as u32,
                    (score.draw_chance() * 1000.0) as u32,
                    (score.lose_chance() * 1000.0) as u32
                ));
            }

            println!(
                "info depth {} seldepth {} {} time {} nodes {} nps {} hashfull {} multipv {} pv {}",
                search_stats.avg_depth(),
                search_stats.max_depth(),
                score_text,
                search_stats.time_passed() as u128,
                search_stats.iters() as u128,
                search_stats.iters() as u128 * 1000 / search_stats.time_passed().max(1) as u128,
                (usage * 1000.0) as u32,
                multi_pv_idx + 1,
                pv_string
            )
        }
    }
    fn print_search_result(&self, mv: Move, score: Score) {
        println!("bestmove {}", mv)
    }
}
