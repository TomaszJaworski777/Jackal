use colored::Colorize;
use console::pad_str;
use spear::{ChessPosition, Move, StringUtils};

use crate::{
    clear_terminal_screen,
    search::Score,
    utils::{heat_color, lerp_color},
    EngineOptions, GameState, SearchLimits, SearchStats,
};

use super::SearchDisplay;

pub struct PrettyPrint {
    start_height: i32,
    max_history_size: usize,
    history: Vec<(u128, String)>,
    last_best_move: Move,
}
#[allow(unused)]
impl SearchDisplay for PrettyPrint {
    fn new(position: &ChessPosition, engine_options: &EngineOptions) -> Self {
        clear_terminal_screen();
        position.board().draw_board();
        println!(" {}    1", "Threads:".bright_black());
        println!(
            " {}  {}MB",
            "Tree Size:".bright_black(),
            engine_options.hash()
        );

        #[cfg(target_os = "linux")]
        let start_height = 14;
        #[cfg(target_os = "windows")]
        let start_height = 13;

        let term_height = term_size::dimensions()
            .expect("Cannot obtain terminal size")
            .1;

        Self {
            start_height,
            max_history_size: term_height - term_height.min(start_height as usize + 16),
            history: Vec::new(),
            last_best_move: Move::NULL,
        }
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
        term_cursor::set_pos(0, self.start_height).expect("Cannot move curser to the position");

        print!("                                                \r");
        println!(
            " {} {}\n",
            "Tree Usage:".bright_black(),
            usage_bar(50, usage)
        );

        print!("                                    \r");
        println!(
            " {} {}",
            "Avg. Depth:".bright_black(),
            search_stats.avg_depth()
        );
        print!("                                    \r");
        println!(
            " {}  {}\n",
            "Max Depth:".bright_black(),
            search_stats.max_depth()
        );

        print!("                                    \r");
        println!(
            " {}      {}",
            "Nodes:".bright_black(),
            StringUtils::large_number_to_string(search_stats.iters() as u128)
        );
        print!("                                    \r");
        println!(
            " {}       {}",
            "Time:".bright_black(),
            StringUtils::time_to_string(search_stats.time_passed() as u128)
        );
        print!("                                    \r");
        let nps = search_stats.iters() as u64 * 1000 / search_stats.time_passed().max(1);
        println!(
            " {}        {}\n",
            "Nps:".bright_black(),
            StringUtils::large_number_to_string(nps as u128)
        );

        print!("                                    \r");
        let score_cp = score.as_cp_f32();
        let score_cp_string = if score_cp >= 0.0 {
            format!("+{:.2}", score_cp)
        } else {
            format!("{:.2}", score_cp)
        };
        println!(
            " {}      {}",
            "Score:".bright_black(),
            heat_color(score_cp_string.as_str(), f32::from(score), 0.0, 1.0)
        );
        print!("                                    \r");
        println!(
            " {}        {}%W {}%D {}%L",
            "WDL:".bright_black(),
            75,
            7,
            13
        );
        print!("                                                                                                             \r");
        let pv_string = pv_to_string::<FINAL>(pv);
        println!(" {}  {}\n", "Best Line:".bright_black(), pv_string);

        println!(" Search History:");
        let start_idx = self.history.len() - self.max_history_size.min(self.history.len());
        
        #[allow(clippy::needless_range_loop)]
        for idx in start_idx..self.history.len() {
            let (timestamp, pv_line) = &self.history[idx];
            println!(
                "  {} -> {}",
                pad_str(
                    &StringUtils::time_to_string(*timestamp),
                    7,
                    console::Alignment::Right,
                    None
                )
                .bright_black(),
                pv_line
            )
        }
        println!("                                                  ");

        if !pv.is_empty() && pv[0] != self.last_best_move {
            self.last_best_move = pv[0];
            self.history
                .push((search_stats.time_passed() as u128, pv_string));
        }
    }

    fn print_search_result(&self, mv: Move, score: Score) {
        println!("Best Move: {}", format!("{mv}").bright_cyan())
    }
}

fn usage_bar(length: usize, fill: f32) -> String {
    let mut result = String::from("[");

    for i in 0..length {
        let percentage = i as f32 / (length - 1) as f32;
        let char = if percentage <= fill {
            heat_color("#", 1.0 - percentage, 0.0, 1.0)
        } else {
            String::from(".")
        };

        result.push_str(&char);
    }

    result.push_str(&format!("] {}%", (fill * 100.0) as usize));
    result
}

fn pv_to_string<const FINAL: bool>(pv: &[Move]) -> String {
    let start_pv_color = (255, 255, 255);
    let end_pv_color = (100, 100, 100);
    let pv_length = if FINAL { pv.len() } else { pv.len().min(15) };
    let rest = pv.len() - pv_length;
    let mut pv_string = String::new();

    for idx in 0..pv_length {
        pv_string.push_str(&lerp_color(
            &format!("{} ", pv[idx]),
            start_pv_color,
            end_pv_color,
            idx as f32 / 14.0,
        ));
    }

    if rest > 0 {
        pv_string.push_str(&lerp_color(
            &format!("({} more...)", rest),
            start_pv_color,
            end_pv_color,
            1.0,
        ));
    }

    pv_string
}
