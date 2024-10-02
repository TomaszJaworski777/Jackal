use spear::ChessPosition;

use crate::search::{tree::Edge, NodeIndex, Score, SearchHelpers};

use super::Mcts;

impl<'a> Mcts<'a> {
    pub(super) fn process_deeper_node<
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
        const ROOT: bool,
    >(
        &self,
        current_node_idx: NodeIndex,
        action_cpy: &Edge,
        current_position: &mut ChessPosition,
        depth: &mut u32,
    ) -> Option<Score> {
        //If current non-root node is terminal or it's first visit, we don't want to go deeper into the tree
        //therefore we just evaluate the node and thats where recursion ends
        let score =
            if !ROOT && (self.tree[current_node_idx].is_termial() || action_cpy.visits() == 0) {
                SearchHelpers::get_node_score::<STM_WHITE, NSTM_WHITE>(
                    current_position,
                    self.tree[current_node_idx].state(),
                )
            } else {
                //On second visit we expand the node, if it wasn't already expanded.
                //This allows us to reduce amount of time we evaluate policy net
                if !self.tree[current_node_idx].has_children() {
                    self.tree[current_node_idx]
                        .expand::<STM_WHITE, NSTM_WHITE, false>(current_position)
                }

                //We then select the best action to evaluate and advance the position to the move of this action
                let best_action_idx = self.tree[current_node_idx].select_action::<ROOT>(
                    &self.tree,
                    current_node_idx,
                    action_cpy.visits(),
                    self.options.cpuct_value(),
                );
                let edge = self.tree.get_edge_clone(current_node_idx, best_action_idx);
                current_position.make_move::<STM_WHITE, NSTM_WHITE>(edge.mv());

                //Process the new action on the tree and obtain it's updated index
                let new_node_idx = self.tree.get_node_index::<NSTM_WHITE, STM_WHITE>(
                    &current_position,
                    edge.node_index(),
                    current_node_idx,
                    best_action_idx,
                )?;
                self.tree.add_thread(current_node_idx, best_action_idx);

                //Descend deeper into the tree
                *depth += 1;
                let nullable_score = self.process_deeper_node::<NSTM_WHITE, STM_WHITE, false>(
                    new_node_idx,
                    &edge,
                    current_position,
                    depth,
                );

                self.tree.remove_thread(current_node_idx, best_action_idx);

                let score = nullable_score?;
                self.tree.add_score_to_edge(current_node_idx, best_action_idx, score);
                self.tree.backpropagate_mates(current_node_idx, self.tree[new_node_idx].state());

                score
            };

        Some(score.reversed())
    }
}
