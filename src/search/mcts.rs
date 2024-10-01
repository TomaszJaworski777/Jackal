use super::{
    print::SearchDisplay,
    search_limits::SearchLimits,
    tree::{Edge, NodeIndex},
    SearchHelpers, SearchStats, Tree,
};
use crate::options::EngineOptions;
use spear::{ChessPosition, Move, Side};
use std::{
    sync::atomic::{AtomicBool, Ordering}, time::Instant
};

pub struct Mcts<'a> {
    root_position: ChessPosition,
    tree: &'a Tree,
    interruption_token: &'a AtomicBool,
    options: &'a EngineOptions,
    stats: &'a SearchStats,
    limits: &'a SearchLimits,
}

impl<'a> Mcts<'a> {
    pub fn new(
        root_position: ChessPosition,
        tree: &'a Tree,
        interruption_token: &'a AtomicBool,
        options: &'a EngineOptions,
        stats: &'a SearchStats,
        limits: &'a SearchLimits,
    ) -> Self {
        Self {
            root_position,
            tree,
            interruption_token,
            options,
            stats,
            limits,
        }
    }

    pub fn search<PRINTER: SearchDisplay>(&self) -> (Move, f64) {
        PRINTER::print_search_start(self.stats, self.options, self.limits);

        //Check if root node is expanded, and if not then expand it
        let root_index = self.tree.root_index();
        if !self.tree[root_index].has_children() {
            let side_to_move = self.root_position.board().side_to_move();
            if side_to_move == Side::WHITE {
                self.tree[root_index].expand::<true, false, true>(&self.root_position)
            } else {
                self.tree[root_index].expand::<false, true, true>(&self.root_position)
            }
        }

        //Start mcts search loop
        if self.root_position.board().side_to_move() == Side::WHITE {
            self.main_loop::<PRINTER, true, false>()
        } else {
            self.main_loop::<PRINTER, false, true>()
        }

        let (best_move, best_score) = self.tree.get_best_move(self.tree.root_index());
        self.stats.update_time_passed();
        PRINTER::print_search_raport(
            self.stats,
            self.options,
            self.limits,
            best_score,
            self.tree[self.tree.root_index()].state(),
            &self.tree.get_pv(),
        );
        PRINTER::print_search_result(best_move, best_score);
        (best_move, best_score)
    }

    fn main_loop<PRINTER: SearchDisplay, const STM_WHITE: bool, const NSTM_WHITE: bool>(&self) {
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
                let (_, best_score) = self.tree.get_best_move(root_index);
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

    fn process_deeper_node<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        current_node_index: NodeIndex,
        action_cpy: &Edge,
        current_position: &mut ChessPosition,
        depth: &mut u32,
    ) -> Option<f32> {

        //If current non-root node is terminal or it's first visit, we don't want to go deeper into the tree
        //therefore we just evaluate the node and thats where recursion ends
        let score = if !ROOT
            && (self.tree[current_node_index].is_termial() || action_cpy.visits() == 0)
        {
            SearchHelpers::get_node_score::<STM_WHITE, NSTM_WHITE>(
                current_position,
                self.tree[current_node_index].state(),
            )
        } else {
            //On second visit we expand the node, if it wasn't already expanded.
            //This allows us to reduce amount of time we evaluate policy net
            if !self.tree[current_node_index].has_children() {
                self.tree[current_node_index].expand::<STM_WHITE, NSTM_WHITE, false>(current_position)
            }

            //We then select the best action to evaluate and advance the position to the move of this action
            let best_action_index = self.tree[current_node_index].select_action::<ROOT>(
                &self.tree,
                current_node_index,
                action_cpy.visits(),
                self.options.cpuct_value(),
            );
            let new_edge_cpy = self
                .tree
                .get_edge_clone(current_node_index, best_action_index);
            current_position.make_move::<STM_WHITE, NSTM_WHITE>(new_edge_cpy.mv());

            //Process the new action on the tree and obtain it's updated index
            let new_node_index = self.tree.get_node_index::<NSTM_WHITE, STM_WHITE>(&current_position, new_edge_cpy.node_index(), current_node_index, best_action_index)?;

            //Descend deeper into the tree
            *depth += 1;
            let score = self.process_deeper_node::<NSTM_WHITE, STM_WHITE, false>(
                new_node_index,
                &new_edge_cpy,
                current_position,
                depth,
            )?;

            self.tree.add_edge_score::<false>(current_node_index, best_action_index, score);

            self.tree.backpropagate_mates(current_node_index, self.tree[new_node_index].state());

            score
        };

        Some(1.0 - score)
    }
}
