use core::f32;

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
        current_node_index: NodeIndex,
        action_cpy: &Edge,
        current_position: &mut ChessPosition,
        depth: &mut u32,
    ) -> Option<Score> {
        //If current non-root node is terminal or it's first visit, we don't want to go deeper into the tree
        //therefore we just evaluate the node and thats where recursion ends
        let score = if !ROOT
            && (self.tree[current_node_index].is_terminal() || action_cpy.visits() == 0)
        {
            SearchHelpers::get_node_score::<STM_WHITE, NSTM_WHITE>(
                current_position,
                self.tree[current_node_index].state(),
                self.tree[current_node_index].key(),
                self.tree
            )
        } else {
            //On second visit we expand the node, if it wasn't already expanded.
            //This allows us to reduce amount of time we evaluate policy net
            if !self.tree[current_node_index].has_children() {
                self.tree[current_node_index].expand::<STM_WHITE, NSTM_WHITE, false>(current_position, self.options)
            }

            //We then select the best action to evaluate and advance the position to the move of this action
            let best_action_index = self.select_action::<ROOT>(
                current_node_index,
                action_cpy.visits(),
                action_cpy.score()
            );
            let new_edge_cpy = self
                .tree
                .get_edge_clone(current_node_index, best_action_index, 2);
            current_position.make_move::<STM_WHITE, NSTM_WHITE>(new_edge_cpy.mv());

            //Process the new action on the tree and obtain it's updated index
            let new_node_index = self.tree.get_node_index::<NSTM_WHITE, STM_WHITE>(
                current_position,
                new_edge_cpy.node_index(),
                current_node_index,
                best_action_index,
            )?;

            //Increase amount of threads visiting this node
            self.tree[new_node_index].inc_threads();

            //Descend deeper into the tree
            *depth += 1;
            let opt_score = self.process_deeper_node::<NSTM_WHITE, STM_WHITE, false>(
                new_node_index,
                &new_edge_cpy,
                current_position,
                depth,
            );

            //When thread leaves the node, decrease the counter
            self.tree[new_node_index].dec_threads();
            let score = opt_score?;

            //Backpropagate the score up the tree
            self.tree
                .add_edge_score(current_node_index, best_action_index, score);

            //Backpropagate mates to assure our engine avoids/follows mating line
            self.tree
                .backpropagate_mates(current_node_index, self.tree[new_node_index].state());

            score
        };

        Some(score.reversed())
    }

    //PUCT formula V + C * P * (N.max(1).sqrt()/n + 1) where N = number of visits to parent node, n = number of visits to a child
    #[inline]
    fn select_action<const ROOT: bool>(
        &self,
        node_idx: NodeIndex,
        parent_visits: u32,
        parent_score: Score
    ) -> usize {
        assert!(self.tree[node_idx].has_children());

        let mut cpuct = if ROOT { 
            self.options.root_cpuct_value() 
        } else { 
            self.options.cpuct_value() 
        };

        let scale = self.options.cpuct_visits_scale() * 128.0;
        cpuct *= 1.0 + ((parent_visits as f32 + scale) / scale).ln();

        let explore_value = cpuct * (self.options.exploration_tau() * (parent_visits.max(1) as f32).ln()).exp();
        self.tree[node_idx].get_best_action_by_key(|action| {
            let visits = action.visits();
            let mut score = if visits == 0 {
                1.0 - f32::from(parent_score)
            } else {
                f32::from(action.score())
            };

            //virtual loss
            let idx = action.node_index();
            if !idx.is_null() {
                let thrds = f64::from(self.tree[idx].threads());
                let v = f64::from(visits);

                if thrds > 0.0 {

                    //score adjusted by the amount of thread visits
                    let s = f64::from(score) * v / (v + thrds);
                    score = s as f32;
                }
            }

            score + (explore_value * action.policy() / (visits as f32 + 1.0))
        })
    }
}
