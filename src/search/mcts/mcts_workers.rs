use std::{sync::atomic::Ordering, thread, time::Instant};

use spear::Move;

use crate::search::print::SearchDisplay;

use super::Mcts;

impl<'a> Mcts<'a> {
    pub(super) fn search_loop<
        PRINTER: SearchDisplay,
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
    >(
        &self,
        printer: &'a mut PRINTER,
    ) {
        loop {
            thread::scope(|s| {
                s.spawn(|| {
                    self.main_loop::<PRINTER, STM_WHITE, NSTM_WHITE>(printer);
                });

                for _ in 0..self.options.threads() - 1 {
                    s.spawn(|| self.worker_loop::<STM_WHITE, NSTM_WHITE>());
                }
            });

            if self.interruption_token.load(Ordering::Relaxed) {
                return;
            }

            self.tree.advance_segments();     
        }
    }

    fn main_loop<
        PRINTER: SearchDisplay,
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
    >(
        &self,
        printer: &'a mut PRINTER,
    ) {

        let mut best_move = Move::NULL;
        let mut best_move_changes = 0;
        let mut previous_score = f32::NEG_INFINITY;

        let mut last_raport_time = Instant::now();
        let mut last_avg_depth = 0;
        loop {
            //Start tree descend
            let mut depth = 0;
            let mut position = self.root_position;
            let root_index = self.tree.root_index();
            let result = self.process_deeper_node::<STM_WHITE, NSTM_WHITE, true, true, false>(
                root_index,
                self.tree.root_edge(),
                &mut position,
                &mut depth,
            );

            let draw_contempt = self.options.draw_contempt();
            if let Some(score) = result {
                self.tree.root_edge().add_score(score);
            } else {
                return;
            }

            //Increment search stats
            self.stats.add_iteration(depth);

            //Interrupt search when root becomes terminal node, so when there is a force mate on board
            if self.tree[root_index].is_terminal() && !self.options.analyse_mode() {
                self.interruption_token.store(true, Ordering::Relaxed)
            }

            //Update timer every few iterations to reduce the slowdown caused by obtaining time
            if self.stats.iters() % 256 == 0 {
                self.stats.update_time_passed();

                //Check hard time limit
                if self.limits.is_hard_time_limit_reached(self.stats, self.options) {
                    self.interruption_token.store(true, Ordering::Relaxed)
                }

                //Update best move
                let new_best_move = self.tree[self.tree.root_index()].get_best_move(self.tree, draw_contempt).0;
                if new_best_move != best_move {
                    best_move = new_best_move;
                    best_move_changes += 1;
                }
            }

            //Check soft time every larger chunk of iterations
            if self.stats.iters() % 16384 == 0 {
                if self.limits.is_soft_time_limit_reached(self.stats, self.options, &mut best_move_changes, &mut previous_score, &self.tree) {
                    self.interruption_token.store(true, Ordering::Relaxed)
                }
            }

            //Check for end of the search
            if self.limits.is_limit_reached(self.stats, self.options) {
                self.interruption_token.store(true, Ordering::Relaxed)
            }

            //Break out of the search
            if self.interruption_token.load(Ordering::Relaxed) {
                return;
            }

            //Draws report when avg_depth increases or if there wasn't any reports for longer than refresh rate
            if self.stats.avg_depth() > last_avg_depth
                || last_raport_time.elapsed().as_secs_f32() > PRINTER::REFRESH_RATE
            {
                last_avg_depth = last_avg_depth.max(self.stats.avg_depth());
                last_raport_time = Instant::now();
                printer.print_search_raport::<false>(
                    self.stats,
                    self.options,
                    self.limits,
                    self.tree.total_usage(),
                    &self.tree.get_pvs(self.options.multi_pv(), draw_contempt)
                )
            }
        }
    }

    fn worker_loop<const STM_WHITE: bool, const NSTM_WHITE: bool>(&self) {
        loop {
            //Start tree descend
            let mut depth = 0;
            let mut position = self.root_position;
            let root_index = self.tree.root_index();
            let result = self.process_deeper_node::<STM_WHITE, NSTM_WHITE, true, true, false>(
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

            //Increment search stats
            self.stats.add_iteration(depth);

            //Interrupt search when root becomes terminal node, so when there is a force mate on board
            if self.tree[self.tree.root_index()].is_terminal() {
                return;
            }

            //Break out of the search
            if self.interruption_token.load(Ordering::Relaxed) {
                return;
            }
        }
    }
}
