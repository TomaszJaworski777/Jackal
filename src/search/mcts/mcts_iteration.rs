use core::f32;

use spear::{ChessPosition, Side};

use crate::search::{networks::PolicyNetwork, tree::Edge, NodeIndex, Score, SearchHelpers};

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
                self.expand::<STM_WHITE, NSTM_WHITE, false>(current_node_index, current_position)
            }

            //We then select the best action to evaluate and advance the position to the move of this action
            let best_action_index = self.select_action::<ROOT>(
                current_node_index,
                action_cpy.visits(),
                self.options.cpuct_value(),
            );
            let new_edge_cpy = self
                .tree
                .get_edge_clone(current_node_index, best_action_index);
            current_position.make_move::<STM_WHITE, NSTM_WHITE>(new_edge_cpy.mv());

            //Process the new action on the tree and obtain it's updated index
            let new_node_index = self.tree.get_node_index::<NSTM_WHITE, STM_WHITE>(
                current_position,
                new_edge_cpy.node_index(),
                current_node_index,
                best_action_index,
            )?;

            //Descend deeper into the tree
            *depth += 1;
            let score = self.process_deeper_node::<NSTM_WHITE, STM_WHITE, false>(
                new_node_index,
                &new_edge_cpy,
                current_position,
                depth,
            )?;

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

    pub fn expand<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        node_idx: NodeIndex,
        position: &ChessPosition,
    ) {
        let mut actions = self.tree[node_idx].actions_mut();

        let mut inputs: Vec<usize> = Vec::with_capacity(32);
        PolicyNetwork::map_policy_inputs::<_, STM_WHITE, NSTM_WHITE>(position.board(), |idx| inputs.push(idx) );

        let vertical_flip = if position.board().side_to_move() == Side::WHITE {
            0
        } else {
            56
        };

        let mut max = f32::NEG_INFINITY;
        let mut total = 0.0;

        //Map moves into actions and set initial policy to 1
        const MULTIPLIER : f32 = 1000.0;
        position
            .board()
            .map_moves::<_, STM_WHITE, NSTM_WHITE>(|mv| {
                let policy = PolicyNetwork.forward(&inputs, mv, vertical_flip);
                actions.push(Edge::new(NodeIndex::from_raw((policy * MULTIPLIER) as u32), mv, 0.0));
                max = max.max(policy);
            });

        for action in actions.iter_mut() {
            let policy = action.node_index().get_raw() as f32 / MULTIPLIER;
            let policy = (policy - max).exp();
            total += policy;
            action.set_node_index(NodeIndex::from_raw((policy * MULTIPLIER) as u32));
        }

        let is_single_action = actions.len() == 1;
        for action in actions.iter_mut() {
            let policy = action.node_index().get_raw() as f32 / MULTIPLIER;
            let policy = if is_single_action { 1.0 } else { policy / total };
            action.update_policy(policy);
            action.set_node_index(NodeIndex::NULL);
        }
    }

    //PUCT formula V + C * P * (N.max(1).sqrt()/n + 1) where N = number of visits to parent node, n = number of visits to a child
    #[inline]
    fn select_action<const ROOT: bool>(
        &self,
        node_idx: NodeIndex,
        parent_visits: u32,
        cpuct: f32,
    ) -> usize {
        assert!(self.tree[node_idx].has_children());

        let explore_value = cpuct * (parent_visits.max(1) as f32).sqrt();
        self.tree[node_idx].get_best_action_by_key(|action| {
            let visits = action.visits();
            let score = if visits == 0 {
                0.5
            } else {
                f32::from(action.score())
            };

            score + (explore_value * action.policy() / (visits as f32 + 1.0))
        })
    }
}
