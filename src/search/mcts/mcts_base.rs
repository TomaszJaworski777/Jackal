use crate::{
    options::EngineOptions,
    search::{print::SearchDisplay, Score},
    SearchLimits, SearchStats, Tree,
};
use spear::{ChessPosition, Move, Piece, Side};
use std::sync::atomic::AtomicBool;

pub struct Mcts<'a> {
    pub(super) root_position: ChessPosition,
    pub(super) tree: &'a Tree,
    pub(super) interruption_token: &'a AtomicBool,
    pub(super) options: &'a EngineOptions,
    pub(super) stats: &'a SearchStats,
    pub(super) limits: &'a SearchLimits,
    pub(super) start_material: i32
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
            start_material: Self::calculate_stm_material(&root_position, root_position.board().side_to_move())
        }
    }

    pub fn search<PRINTER: SearchDisplay>(&self) -> (Move, Score) {
        let mut printer = PRINTER::new(&self.root_position, self.options, &self.tree);

        //Check if root node is expanded, and if not then expand it
        let root_index = self.tree.root_index();
        let side_to_move = self.root_position.board().side_to_move();
        if !self.tree[root_index].has_children() {
            if side_to_move == Side::WHITE {
                self.tree[root_index].expand::<true, false, true>(&self.root_position, self.options)
            } else {
                self.tree[root_index].expand::<false, true, true>(&self.root_position, self.options)
            }
        } else {
            if side_to_move == Side::WHITE {
                self.tree[root_index].recalculate_policy::<true, false, true>(&self.root_position, self.options)
            } else {
                self.tree[root_index].recalculate_policy::<false, true, true>(&self.root_position, self.options)
            }
        }

        //Start mcts search loop
        if self.root_position.board().side_to_move() == Side::WHITE {
            self.search_loop::<PRINTER, true, false>(&mut printer)
        } else {
            self.search_loop::<PRINTER, false, true>(&mut printer)
        }

        //At the end of the search print the last search update raport and then print
        //end of search message containing search result
        let (best_move, best_score) = self.tree[self.tree.root_index()].get_best_move(self.tree);
        self.stats.update_time_passed();
        printer.print_search_raport::<true>(
            self.stats,
            self.options,
            self.limits,
            self.tree.total_usage(),
            &self.tree.get_pvs(self.options.multi_pv())
        );
        printer.print_search_result(best_move, best_score);
        (best_move, best_score)
    }

    pub(super) fn calculate_stm_material(position: &ChessPosition, side: Side) -> i32 {
        const PIECE_VALUES: [i32; 5] = [100, 300, 300, 500, 900];
        let mut result = 0;
    
        for piece in Piece::PAWN.get_raw()..=Piece::QUEEN.get_raw() {
            let piece_mask = if side == Side::WHITE {
                position.board().get_piece_mask_for_side::<true>(Piece::from_raw(piece))
            } else {
                position.board().get_piece_mask_for_side::<false>(Piece::from_raw(piece))
            };
            result += piece_mask.pop_count() as i32 * PIECE_VALUES[piece as usize];
        }
        result
    }
}
