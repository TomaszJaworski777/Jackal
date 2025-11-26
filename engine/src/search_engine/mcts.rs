use std::{thread, time::Instant};

use chess::Move;

use crate::{
    search_engine::{search_limits::TimeManager, SearchLimits, SearchStats},
    SearchEngine, SearchReport,
};

mod iteration;

impl SearchEngine {
    pub(super) fn mcts<Display: SearchReport>(&self, search_limits: &SearchLimits) -> SearchStats {
        let castle_mask = self
            .root_position()
            .board()
            .castle_rights()
            .get_castle_mask();

        let search_stats = SearchStats::new(0);

        let mut search_report_timer = Instant::now();
        let mut max_avg_depth = 0;

        let mut last_best_move = None;
        let mut best_move_changes = 0;

        loop 
        {
            let mut time_manager = search_limits.time_manager();

            thread::scope(|s| {
                s.spawn(|| {
                    self.main_loop::<Display>(&search_stats, &search_limits, &mut time_manager, &castle_mask, &mut search_report_timer, &mut max_avg_depth, &mut last_best_move, &mut best_move_changes);
                });

                for _ in 0..(self.options().threads() - 1) {
                    s.spawn(|| {
                        self.worker_loop(&search_stats, &search_limits, &castle_mask)
                    });
                }
            });

            if self.is_search_interrupted() {
                break;
            }

            self.tree().swap_half();
        }

        search_stats
    }

    fn main_loop<Display: SearchReport>(
        &self,
        search_stats: &SearchStats,
        search_limits: &SearchLimits,
        time_manager: &mut TimeManager,
        castle_mask: &[u8; 64],
        search_report_timer: &mut Instant,
        max_avg_depth: &mut u64,
        last_best_move: &mut Option<Move>,
        best_move_changes: &mut usize
    ) -> Option<()> {
        #[allow(unused)]
        let mut latest_kld_distribution: Vec<u32> = Vec::new();

        while !self.is_search_interrupted() {
            self.search_step(search_stats, search_limits, castle_mask)?;

            if search_stats.avg_depth() > *max_avg_depth || search_report_timer.elapsed().as_secs_f64() > (1.0 / Display::refresh_rate_per_second()) {
                Display::search_report(search_limits, search_stats, self);
                *search_report_timer = Instant::now();
                *max_avg_depth = search_stats.avg_depth().max(*max_avg_depth);
            }

            let draw_score = self.options().draw_score() as f64 / 100.0;
            let best_move = self.tree()[self.tree().select_best_child(self.tree().root_index(), draw_score).unwrap()].mv();
            if let Some(last_move) = last_best_move {
                if *last_move != best_move {
                    *best_move_changes += 1;
                }
            }

            *last_best_move = Some(best_move);

            #[cfg(feature = "datagen")] { 
                if self.kld_limit(&mut latest_kld_distribution, self.options().kld_min()) {
                    self.interrupt_search();
                    break;
                }
            }

            if search_stats.iterations() % 128 != 0 {
                continue;
            }

            if time_manager.hard_limit_reached(search_stats) {
                self.interrupt_search();
                break;
            }

            if search_stats.iterations() % 4096 != 0 {
                continue;
            }

            if time_manager.soft_limit_reached(search_stats, self.tree(), self.options(), *best_move_changes) {
                self.interrupt_search();
                break;
            }

            if search_stats.iterations() % 16384 != 0 {
                continue;
            }

            *best_move_changes = 0;
        }

        Some(())
    }

    fn worker_loop(
        &self,
        search_stats: &SearchStats,
        search_limits: &SearchLimits,
        castle_mask: &[u8; 64],
    ) -> Option<()> {
        while !self.is_search_interrupted() {
            self.search_step(search_stats, search_limits, castle_mask)?;
        }

        Some(())
    }

    fn search_step(        
        &self,         
        search_stats: &SearchStats,
        search_limits: &SearchLimits,
        castle_mask: &[u8; 64],
    ) -> Option<()> {
        let mut depth = 0.0;
        let mut position = *self.root_position();

        self.perform_iteration::<true>(self.tree().root_index(), &mut position, &mut depth, castle_mask)?;

        search_stats.add_iteration(depth as u64);

        if self.tree().root_node().is_terminal() {
            self.interrupt_search();
        }

        if search_limits.is_limit_reached(search_stats) {
            self.interrupt_search();
        }

        Some(())
    }

    #[allow(unused)]
    fn kld_limit(&self, old_distribution: &mut Vec<u32>, kld_min: f64) -> bool {
        let root = self.tree().root_node();

        let mut new_distribution = Vec::with_capacity(root.children_count());
        root.map_children(|child_idx| new_distribution.push(self.tree()[child_idx].visits()));

        if let Some(kld_gain) = calculate_kld_gain(&new_distribution, old_distribution) {
            if kld_gain < kld_min {
                return true;
            }
        }

        *old_distribution = new_distribution;

        false
    }
}

fn calculate_kld_gain(current_distribution: &[u32], old_distribution: &[u32]) -> Option<f64> {
    let current_sum = current_distribution.iter().sum::<u32>() as f64;
    let old_sum = old_distribution.iter().sum::<u32>() as f64;

    if old_sum == 0.0 {
        return None;
    }

    let mut result = 0.0;
    for (&a, &b) in current_distribution.iter().zip(old_distribution) {
        if b == 0 {
            return None;
        }

        let q = a as f64 / current_sum;
        let p = b as f64 / old_sum;

        result += p * (p / q).ln();
    };

    Some(result / (current_sum - old_sum))
}
