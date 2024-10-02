use std::{sync::atomic::Ordering, thread, time::Instant};

use crate::search::print::SearchDisplay;

use super::Mcts;

impl<'a> Mcts<'a> {
    pub(super) fn execute<
        PRINTER: SearchDisplay,
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
    >(
        &self,
    ) {
        let mut last_raport_time = Instant::now();
        let mut last_avg_depth = 0;

        loop {
            thread::scope(|s| {
                s.spawn(|| {
                    if self.run_main_loop::<PRINTER ,STM_WHITE, NSTM_WHITE>(&mut last_raport_time, &mut last_avg_depth) {
                        self.interruption_token.store(true, Ordering::Relaxed)
                    }
                });

                for _ in 0..(self.threads - 1) {
                    s.spawn(|| self.run_worker::<STM_WHITE, NSTM_WHITE>());
                }
            });

            if self.interruption_token.load(Ordering::Relaxed) {
                break;
            }

            self.tree.advance_segments();
        }
    }

    fn run_main_loop<PRINTER: SearchDisplay, const STM_WHITE: bool, const NSTM_WHITE: bool>(&self, last_raport_time: &mut Instant, last_avg_depth: &mut u32) -> bool{
        loop {
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
                self.tree.root_edge().add_score(score);
            } else {
                return false;
            }

            //Increment search stats
            self.stats.add_iteration(depth, true);

            //Interrupt search when root becomes terminal node, so when there is a force mate on board
            if self.tree[root_index].is_termial() {
                return true;
            }

            //Update timer every few iterations to reduce the slowdown caused by obtaining time
            if self.stats.iters() % 128 == 0 {
                self.stats.update_time_passed()
            }

            //Check for end of the search
            if self.limits.is_limit_reached(self.stats, self.options) {
                return true;
            }
                    
            if self.interruption_token.load(Ordering::Relaxed) {
                return true;
            }

            //Draws report when avg_depth increases or if there wasnt any report for 1s
            if self.stats.avg_depth() > *last_avg_depth || last_raport_time.elapsed().as_secs_f32() > 1.0 {
                *last_avg_depth = self.stats.avg_depth().max(*last_avg_depth);
                *last_raport_time = Instant::now();
                let root_node = &self.tree[self.tree.root_index()];
                let (_, best_score) = root_node.get_best_move(&self.tree);
                PRINTER::print_search_raport(
                    self.stats,
                    self.options,
                    self.limits,
                    best_score,
                    root_node.state(),
                    &self.tree.get_pv(),
                )
            }
        }
    }

    fn run_worker<const STM_WHITE: bool, const NSTM_WHITE: bool>(&self) {
        loop {
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
                self.tree.root_edge().add_score(score);
            } else {
                return;
            }

            self.stats.add_iteration(depth, false);
                    
            if self.tree[root_index].is_termial() {
                return;
            }

            if self.interruption_token.load(Ordering::Relaxed) {
                return;
            }
        }
    }
}
