use colored::Colorize;
use console::pad_str;
use spear::{ChessPosition, Move, StringUtils};

use crate::{
    clear_terminal_screen,
    color_config::{ColorConfig, Colored},
    search::Score,
    utils::{heat_color, lerp_color},
    EngineOptions, GameState, SearchLimits, SearchStats, Tree,
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
    const REFRESH_RATE: f32 = 0.05;

    fn new(position: &ChessPosition, engine_options: &EngineOptions, tree: &Tree) -> Self {
        clear_terminal_screen();
        position.board().draw_board();
        println!(" {}    {}", "Threads:".label(), engine_options.threads());
        println!(
            " {}  {}B",
            "Tree Size:".label(),
            bytes_to_string(tree.tree_size_in_bytes as u128)
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
            max_history_size: term_height - term_height.min(start_height as usize + 19),
            history: Vec::new(),
            last_best_move: Move::NULL,
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn print_search_raport<const FINAL: bool>(
        &mut self,
        search_stats: &SearchStats,
        engine_options: &EngineOptions,
        search_limits: &SearchLimits,
        usage: f32,
        pvs: &Vec<(Score, GameState, Vec<Move>)>,
    ) {
        let (mut score, state, pv) = &pvs[0];
        score = match *state {
            GameState::Drawn => Score::DRAW,
            GameState::Won(x) => Score::LOSE,
            GameState::Lost(x) => Score::WIN,
            _ => score,
        };

        term_cursor::set_pos(0, self.start_height).expect("Cannot move curser to the position");

        print!("                                                \r");
        println!(" {} {}\n", "Tree Usage:".label(), usage_bar(50, usage));

        print!("                                    \r");
        println!(" {} {}", "Avg. Depth:".label(), search_stats.avg_depth());
        print!("                                    \r");
        println!(" {}  {}\n", "Max Depth:".label(), search_stats.max_depth());

        print!("                                    \r");
        println!(
            " {}      {}",
            "Nodes:".label(),
            StringUtils::large_number_to_string(search_stats.iters() as u128)
        );
        print!("                                    \r");
        println!(
            " {}       {}",
            "Time:".label(),
            StringUtils::time_to_string(search_stats.time_passed() as u128)
        );
        print!("                                    \r");
        let nps = search_stats.iters() as u64 * 1000 / search_stats.time_passed().max(1);
        println!(
            " {}        {}\n",
            "Nps:".label(),
            StringUtils::large_number_to_string(nps as u128)
        );

        print!("                                    \r");
        let score_cp = score.as_cp_f32();
        let mut score_cp_string = match *state {
            GameState::Drawn => "+0.0".to_string(),
            GameState::Won(x) => format!("-M{}", ((x + 1) as f32 / 2.0).ceil() as u32),
            GameState::Lost(x) => format!("+M{}", ((x + 1) as f32 / 2.0).ceil() as u32),
            _ => {
                if score_cp >= 0.0 {
                    format!("+{:.2}", score_cp)
                } else {
                    format!("{:.2}", score_cp)
                }
            }
        };

        println!(
            " {}      {}",
            "Score:".label(),
            heat_color(score_cp_string.as_str(), score.single(), 0.0, 1.0)
        );
        print!("                                                                                                             \r");
        println!(
            " {}        {}",
            "Win:".label(),
            color_bar(50, score.win_chance(), ColorConfig::WIN_COLOR)
        );
        print!("                                                                                                             \r");
        println!(
            " {}       {}",
            "Draw:".label(),
            color_bar(50, score.draw_chance(), ColorConfig::DRAW_COLOR)
        );
        print!("                                                                                                             \r");
        println!(
            " {}       {}",
            "Lose:".label(),
            color_bar(50, score.lose_chance(), ColorConfig::LOSE_COLOR)
        );
        print!("                                                                                                             \r");
        let pv_string = pv_to_string::<FINAL>(pv);

        if FINAL {
            for _ in 0..5 {
                println!("                                                                                                             ", );
            }

            term_cursor::set_pos(0, self.start_height + 13)
                .expect("Cannot move curser to the position");
        }

        println!(" {}  {}", "Best Line:".label(), pv_string);
        println!("                                                                                                             ", );
        println!(" Search History:");
        let start_idx = self.history.len() - self.max_history_size.min(self.history.len());

        for idx in start_idx..self.history.len() {
            let (timestamp, pv_line) = &self.history[idx];
            print!("                                                                                                                    \r");
            println!(
                "  {} -> {}",
                pad_str(
                    &StringUtils::time_to_string(*timestamp),
                    7,
                    console::Alignment::Right,
                    None
                )
                .to_string()
                .label(),
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
        println!("Best Move: {}", format!("{mv}").highlight())
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

fn color_bar(length: usize, fill: f32, (r, g, b): (u8, u8, u8)) -> String {
    let mut result = String::from("[");

    for i in 0..length {
        let percentage = i as f32 / (length - 1) as f32;
        let char = if percentage <= fill && fill > 0.0 {
            "#".truecolor(r, g, b).to_string()
        } else {
            String::from(".")
        };

        result.push_str(&char);
    }

    result.push_str(&format!("] {}%", (fill * 100.0) as usize));
    result
}

#[allow(clippy::needless_range_loop)]
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
            idx as f32 / (pv_length - 1).max(14) as f32,
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

fn bytes_to_string(number: u128) -> String {
    match number {
        0..=1023 => format!("{number}"),
        1024..=1_048_575 => format!("{:.2}K", number as f64 / 1024.0),
        1_048_576..=1_073_741_823 => format!("{:.2}M", number as f64 / 1_048_576.0),
        1_073_741_824.. => format!("{:.2}G", number as f64 / 1_073_741_824.0),
    }
}
