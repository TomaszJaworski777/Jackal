use std::{sync::atomic::Ordering, time::Instant};

use crate::search::print::SearchDisplay;

use super::Mcts;

impl<'a> Mcts<'a> {
    pub(super) fn main_loop<PRINTER: SearchDisplay, const STM_WHITE: bool, const NSTM_WHITE: bool>(&self) {
        let mut last_raport_time = Instant::now();
        let mut last_avg_depth = 0;
        loop {
            //Start tree descend
            let mut depth = 0;
            let mut position = self.root_position;
            let root_index = self.tree.root_index();
            let result = self.process_deeper_node::<STM_WHITE, NSTM_WHITE, true>(
                root_index,
                self.tree.root_edge(),
                &mut position,
                &mut depth,
            );

            if let Some(score) = result {
                self.tree.add_edge_score::<true>(self.tree.root_index(), 0, score);
            } else {
                self.tree.advance_segments();
                continue;
            }

            //Increment search stats
            self.stats.add_iteration(depth);

            //Interrupt search when root becomes terminal node, so when there is a force mate on board
            if self.tree[root_index].is_termial() {
                self.interruption_token.store(true, Ordering::Relaxed)
            }

            //Update timer every few iterations to reduce the slowdown caused by obtaining time
            if self.stats.iters() % 128 == 0 {
                self.stats.update_time_passed()
            }

            //Check for end of the search
            if self.limits.is_limit_reached(self.stats, self.options) {
                self.interruption_token.store(true, Ordering::Relaxed)
            }

            //Break out of the search
            if self.interruption_token.load(Ordering::Relaxed) {
                break;
            }

            //draws report when avg_depth increases or if there wasnt any report for 1s
            if self.stats.avg_depth() > last_avg_depth
                || last_raport_time.elapsed().as_secs_f32() > 1.0
            {
                last_avg_depth = last_avg_depth.max(self.stats.avg_depth());
                last_raport_time = Instant::now();
                let (_, best_score) = self.tree[root_index].get_best_move(&self.tree);
                PRINTER::print_search_raport(
                    self.stats,
                    self.options,
                    self.limits,
                    best_score,
                    self.tree[self.tree.root_index()].state(),
                    &self.tree.get_pv(),
                )
            }
        }
    }
}