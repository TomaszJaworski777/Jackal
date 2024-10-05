use colored::Colorize;
use console::pad_str;
use spear::{Move, StringUtils};

use crate::{search::Score, utils::{heat_color, lerp_color}, EngineOptions, GameState, SearchLimits, SearchStats};

use super::SearchDisplay;

pub struct PrettyPrint;
#[allow(unused)]
impl SearchDisplay for PrettyPrint {
    fn new() -> Self {
        PrettyPrint
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
        let avg_depth = search_stats.avg_depth().to_string().bright_cyan().bold().to_string();
        let avg_depth = pad_str(
            &avg_depth,
            2,
            console::Alignment::Right,
            None);

        let max_depth = search_stats.max_depth().to_string().bright_cyan().bold().to_string();
        let max_depth = pad_str(
            &max_depth,
            3,
            console::Alignment::Left,
            None);

        let score_cp = score.as_cp_f32();
        let score_cp_string = if score_cp >= 0.0 {
            format!("+{:.2}", score_cp)
        } else {
            format!("{:.2}", score_cp)
        };
        let score = heat_color(score_cp_string.as_str(), f32::from(score), 0.0, 1.0);
        let score = pad_str(
            &score,
            5,
            console::Alignment::Right,
            None
        );

        let time_passed = format!("{:.2}", search_stats.time_passed() as f32 / 1000.0).truecolor(230, 230, 230).bold().to_string();
        let time_passed = pad_str(
            &time_passed, 
            6, 
            console::Alignment::Right, 
            None);

        let nodes = StringUtils::large_number_to_string(search_stats.iters() as u128).truecolor(230, 230, 230).bold().to_string();
        let nodes = pad_str(
            &nodes, 
            7, 
            console::Alignment::Right, 
            None);

        let nps = search_stats.iters() as u64 * 1000 / search_stats.time_passed().max(1);
        let nps = StringUtils::large_number_to_string(nps as u128).truecolor(230, 230, 230).bold().to_string();
        let nps = pad_str(
            &nps, 
            7, 
            console::Alignment::Right, 
            None);

        let usage = heat_color(&format!("{}%", (usage * 100.0) as u8), 1.0 - usage, 0.0, 1.0);
        let usage = pad_str(
            &usage, 
            3, 
            console::Alignment::Right, 
            None);

        let start_pv_color = (255,255,255);
        let end_pv_color = (128,128,128);
        let pv_length = if FINAL {
            pv.len()
        } else {
            pv.len().min(6)
        };
        let rest = pv.len() - pv_length;
        let mut pv_string = String::new();

        for idx in 0..pv_length {
            pv_string.push_str(&lerp_color(&format!("{} ", pv[idx]), start_pv_color, end_pv_color, (idx as f32 / pv_length as f32).min(1.0)));
        }

        if rest > 0 {
            pv_string.push_str(&lerp_color(&format!("({} more...)", rest), start_pv_color, end_pv_color, 1.0));
        }

        let mut raport = format!("{}/{}   {} [75%W 5%D 10%L]   {} seconds   {} nodes   {} n/s   {} usage   {}", avg_depth, max_depth, score, time_passed, nodes, nps, usage, pv_string);
        println!("{}", raport.bright_black())
    }

    fn print_search_result(&self, mv: Move, score: Score) {
        println!("bestmove {}", format!("{mv}").bright_cyan())
    }
}