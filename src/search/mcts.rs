use std::sync::atomic::AtomicBool;

use spear::{ChessPosition, Move, Side};

use crate::options::EngineOptions;

use super::{print::SearchPrinter, search_limits::SearchLimits, SearchStats, SearchTree};

pub struct Mcts<'a> {
    root_position: ChessPosition,
    tree: &'a SearchTree,
    interruption_token: &'a AtomicBool,
    options: &'a EngineOptions,
    stats: &'a SearchStats,
    limits: &'a SearchLimits,
}

impl<'a> Mcts<'a> {
    pub fn new(
        root_position: ChessPosition,
        tree: &'a SearchTree,
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

    pub fn search<PRINTER: SearchPrinter>(&self) -> (Move, f32) {
        PRINTER::print_search_start(&self.stats, &self.options, &self.limits);

        //Check if root node is expanded, and if not then expand it
        let root_index = self.tree.root_index();
        if !self.tree[root_index].is_expanded() {
            let side_to_move = self.root_position.board().side_to_move();
            if side_to_move == Side::WHITE {
                self.tree[root_index].expand::<true, false>()
            } else {
                self.tree[root_index].expand::<false, true>()
            }
        }

        (Move::NULL, 0.0)
    }

    fn process_deeper_node<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        current_node_index: i32,
        current_position: &ChessPosition,
        depth: u32,
    ) -> f32 {
        0.0
    }
}
