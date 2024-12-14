use crate::options::EngineOptions;

use super::{SearchStats, Tree};

#[derive(Default)]
pub struct SearchLimits {
    time_remaining: Option<u64>,
    increment: Option<u64>,
    moves_to_go: Option<u32>,
    move_time: Option<u64>,
    max_depth: Option<u32>,
    max_iters: Option<u32>,
    infinite: bool,
    game_ply: u32,
    soft_limit: Option<u64>,
    hard_limit: Option<u64>,
}
impl SearchLimits {
    pub fn new(game_ply: u32) -> Self {
        Self {
            time_remaining: None,
            increment: None,
            moves_to_go: None,
            move_time: None,
            max_depth: None,
            max_iters: None,
            infinite: false,
            game_ply,
            soft_limit: None,
            hard_limit: None,
        }
    }

    pub fn add_time_remaining(&mut self, time_remaining: u64) {
        self.time_remaining = Some(time_remaining);
    }

    pub fn add_increment(&mut self, increment: u64) {
        self.increment = Some(increment);
    }

    pub fn add_moves_to_go(&mut self, moves_to_go: u32) {
        self.moves_to_go = Some(moves_to_go);
    }

    pub fn add_move_time(&mut self, move_time: u64) {
        self.move_time = Some(move_time);
    }

    pub fn add_depth(&mut self, depth: u32) {
        self.max_depth = Some(depth);
    }

    pub fn add_iters(&mut self, iters: u32) {
        self.max_iters = Some(iters);
    }

    pub fn go_infinite(&mut self) {
        self.infinite = true;
    }

    pub fn calculate_time_limits(&mut self) {
        if let Some(time) = self.time_remaining {
            let (soft, hard) =
                Self::search_time(time, self.increment, self.moves_to_go, self.game_ply);
            self.soft_limit = Some(soft);
            self.hard_limit = Some(hard);
        }
    }

    pub fn is_limit_reached(&self, search_stats: &SearchStats, _options: &EngineOptions) -> bool {
        if self.infinite {
            return false;
        }

        if let Some(max_depth) = self.max_depth {
            if search_stats.avg_depth() >= max_depth {
                return true;
            }
        }

        if let Some(max_iters) = self.max_iters {
            if search_stats.iters() >= max_iters {
                return true;
            }
        }

        if let Some(time) = self.move_time {
            if search_stats.time_passed() >= time {
                return true;
            }
        }

        false
    }

    pub fn is_hard_time_limit_reached(
        &self,
        search_stats: &SearchStats,
        options: &EngineOptions,
    ) -> bool {
        if let Some(hard_limit) = self.hard_limit {
            let time_passed = search_stats.time_passed() + options.move_overhead() as u64;
            if time_passed >= hard_limit {
                return true;
            }
        }

        false
    }

    pub fn is_soft_time_limit_reached(
        &self,
        search_stats: &SearchStats,
        options: &EngineOptions,
        best_move_changes: &mut i32,
        previous_score: &mut f32,
        tree: &Tree,
    ) -> bool {
        if let Some(soft_limit) = self.soft_limit {
            let time_passed = search_stats.time_passed() + options.move_overhead() as u64;

            let best_move_score = tree[tree.root_index()].get_best_move(tree, options.draw_contempt()).1.as_cp_f32();
            let eval_diff = if *previous_score == f32::NEG_INFINITY {
                0.0
            } else {
                *previous_score - best_move_score
            };

            let falling_eval = (1.0 + eval_diff * 0.05).clamp(0.60, 1.80);
            let best_move_instability =
                (1.0 + (*best_move_changes as f32 * 0.3).ln_1p()).clamp(1.0, 3.2);

            let best_action_index = tree[tree.root_index()].get_best_action(tree, options.draw_contempt());
            let best_action = tree.get_edge_clone(tree.root_index(), best_action_index);
            let nodes_effort = best_action.visits() as f32 / search_stats.iters() as f32;
            let best_move_visits =
                (2.5 - ((nodes_effort + 0.3) * 0.55).ln_1p() * 4.0).clamp(0.55, 1.50);

            let total_limit =
                (soft_limit as f32 * falling_eval * best_move_instability * best_move_visits)
                    as u64;

            if time_passed >= total_limit {
                return true;
            }

            *best_move_changes = 0;
            *previous_score = if *previous_score == f32::NEG_INFINITY {
                best_move_score
            } else {
                (best_move_score + *previous_score) / 2.0
            };
        }

        false
    }

    fn search_time(
        time: u64,
        increment: Option<u64>,
        moves_to_go: Option<u32>,
        game_ply: u32,
    ) -> (u64, u64) {
        let inc = increment.unwrap_or_default();

        if let Some(mtg) = moves_to_go {
            let time = ((time + inc) as f64 / mtg as f64) as u64;
            return (time, time);
        }

        let mtg = 30;

        let time_left = (time + inc * (mtg - 1) - 10 * (2 + mtg)).max(1) as f64;
        let log_time = (time_left / 1000.0).log10();

        let soft_constant = (0.0048 + 0.00032 * log_time).min(0.0060);
        let soft_scale = (0.0125 + (game_ply as f64 + 2.5).sqrt() * soft_constant)
            .min(0.25 * time as f64 / time_left);

        let hard_constant = (3.39 + 3.01 * log_time).max(2.93);
        let hard_scale = (hard_constant + game_ply as f64 / 12.0).min(4.00);

        let bonus = if game_ply <= 10 {
            1.0 + (11.0 - game_ply as f64).log10() * 0.5
        } else {
            1.0
        };

        let soft_time = (soft_scale * bonus * time_left) as u64;
        let hard_time = (hard_scale * soft_time as f64).min(time as f64 * 0.850) as u64;

        (soft_time, hard_time)
    }
}
