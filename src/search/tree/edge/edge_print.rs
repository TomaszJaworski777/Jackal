use crate::{search::Score, utils::heat_color, GameState};

use super::Edge;

use colored::Colorize;
use console::pad_str;

impl Edge { 
    pub fn print<const ROOT: bool>(
        &self,
        lowest_policy: f32,
        highest_policy: f32,
        state: GameState,
        flip_score: bool,
    ) {
        let terminal_string = match state {
            GameState::Drawn => "   terminal draw".white().bold().to_string(),
            GameState::Lost(x) => format!("   terminal lose in {}", x)
                .white()
                .bold()
                .to_string(),
            GameState::Won(x) => format!("   terminal win in {}", x)
                .white()
                .bold()
                .to_string(),
            _ => "".to_string(),
        };

        let index_text = if ROOT {
            "root".bright_cyan().to_string()
        } else {
            format!(
                "{}> {}",
                pad_str(
                    self.node_index()
                        .to_string()
                        .bright_cyan()
                        .to_string()
                        .as_str(),
                    12,
                    console::Alignment::Right,
                    None
                ),
                pad_str(
                    self.mv().to_string().bright_cyan().to_string().as_str(),
                    5,
                    console::Alignment::Right,
                    None
                )
            )
        };

        let score = if flip_score {
            self.score().reversed()
        } else {
            self.score()
        };

        let score = if self.visits() == 0 { Score::DRAW } else { score };
        let score_cp = score.as_cp_f32();
        let score_cp_string = if score_cp >= 0.0 {
            format!("+{:.2}", score_cp)
        } else {
            format!("{:.2}", score_cp)
        };

        println!(
            "{}",
            format!(
                "{}   {} score   {} visits   {} policy{}",
                index_text,
                pad_str(
                    heat_color(score_cp_string.as_str(), f32::from(score), 0.0, 1.0).as_str(),
                    5,
                    console::Alignment::Right,
                    None
                ),
                pad_str(
                    self.visits()
                        .to_string()
                        .bold()
                        .white()
                        .to_string()
                        .as_str(),
                    8,
                    console::Alignment::Right,
                    None
                ),
                pad_str(
                    heat_color(
                        format!("{:.2}%", self.policy() * 100.0).as_str(),
                        self.policy(),
                        lowest_policy,
                        highest_policy
                    )
                    .as_str(),
                    4,
                    console::Alignment::Right,
                    None
                ),
                terminal_string
            )
            .bright_black()
        )
    }
}
