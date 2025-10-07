use chess::ChessPosition;

use crate::{search_engine::tree::NodeIndex, GameState, SearchEngine, WDLScore};

mod select;
mod simulate;
mod backpropagate;

impl SearchEngine {
    pub(super) fn perform_iteration<const ROOT: bool>(
        &self,
        node_idx: NodeIndex,
        position: &mut ChessPosition,
        depth: &mut f64,
        castle_mask: &[u8; 64],
    ) -> Option<WDLScore> { 
        let hash = position.board().hash();
        let node = &self.tree()[node_idx];

        let mut selected_child_idx = None;

        let score = if !ROOT && (node.is_terminal() || node.visits() == 0) {
            self.simulate(node_idx, position, *depth)
        } else {
            *depth += 1.0;

            if node.children_count() == 0 {
                self.tree().expand_node(node_idx, *depth, position.board(), self.options())?
            }

            self.tree().update_node(node_idx)?;

            let new_idx = self.select(node_idx, *depth);

            selected_child_idx = Some(new_idx);

            let old_side = position.board().side();

            let mv = self.tree()[new_idx].mv();
            position.make_move(mv, castle_mask);

            self.tree().inc_threads(new_idx, 1);

            let lock = if self.tree()[new_idx].visits() == 0 {
                Some(node.children_index_mut())
            } else {
                None
            };

            let old_position = position.clone();
            //let base_score = self.tree()[node_idx].base_score();

            let score = self.perform_iteration::<false>(new_idx, position, depth, castle_mask);

            drop(lock);

            self.tree().dec_threads(new_idx, 1);

            let score = score?;

            if !self.tree()[new_idx].is_terminal() {
                self.tree().butterfly_history().update_entry(old_side, mv, score, self.options());
            }

            self.subtree_bias().update(score.reversed().single() as f32, self.tree()[new_idx].base_score(), &old_position);

            score
        }.reversed();

        self.backpropagate(node_idx, selected_child_idx, score, hash);

        Some(score)
    }
}