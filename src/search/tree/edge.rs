use std::sync::atomic::{AtomicI16, AtomicI32, AtomicU32, Ordering};

use colored::Colorize;
use console::pad_str;

use crate::utils::heat_color;
use spear::Move;

use super::GameState;

pub struct Edge {
    node_index: AtomicI32,
    mv: Move,
    policy: AtomicI16,
    visits: AtomicU32,
    score: AtomicU32,
}

impl Clone for Edge {
    fn clone(&self) -> Self {
        Self {
            node_index: AtomicI32::new(self.index()),
            mv: self.mv(),
            policy: AtomicI16::new(self.policy.load(Ordering::Relaxed)),
            visits: AtomicU32::new(self.visits()),
            score: AtomicU32::new(self.score.load(Ordering::Relaxed)),
        }
    }
}

impl Edge {
    pub fn new(node_index: i32, mv: Move, policy: f32) -> Self {
        Self {
            node_index: AtomicI32::new(node_index),
            mv,
            policy: AtomicI16::new((policy * f32::from(i16::MAX)) as i16),
            visits: AtomicU32::new(0),
            score: AtomicU32::new(0),
        }
    }

    #[inline]
    pub fn index(&self) -> i32 {
        self.node_index.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn set_index(&self, index: i32) {
        self.node_index.store(index, Ordering::Relaxed)
    }

    #[inline]
    pub fn mv(&self) -> Move {
        self.mv
    }

    #[inline]
    pub fn policy(&self) -> f32 {
        f32::from(self.policy.load(Ordering::Relaxed)) / f32::from(i16::MAX)
    }

    #[inline]
    pub fn visits(&self) -> u32 {
        self.visits.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn score(&self) -> f64 {
        f64::from(self.score.load(Ordering::Relaxed)) / f64::from(u32::MAX)
    }

    #[inline]
    pub fn add_score(&self, score: f32) {
        let score = f64::from(score);
        let previous_visits = self.visits.fetch_add(1, Ordering::Relaxed) as f64;
        let new_score = (self.score() * previous_visits + score) / (previous_visits + 1.0);
        self.score
            .store((new_score * f64::from(u32::MAX)) as u32, Ordering::Relaxed)
    }

    #[inline]
    pub fn update_policy(&self, new_policy: f32) {
        self.policy
            .store((new_policy * f32::from(i16::MAX)) as i16, Ordering::Relaxed)
    }

    pub fn print<const ROOT: bool>(
        &self,
        lowest_policy: f32,
        highest_policy: f32,
        state: GameState,
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
                "{}. {}",
                pad_str(
                    self.index().to_string().bright_cyan().to_string().as_str(),
                    6,
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

        println!(
            "{}",
            format!(
                "{}   {} score   {} visits   {} policy{}",
                index_text,
                pad_str(
                    heat_color(
                        format!("{:.2}", self.score()).as_str(),
                        self.score() as f32,
                        0.25,
                        0.75
                    )
                    .as_str(),
                    4,
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
