use spear::ChessPosition;

use crate::{search::tree::Edge, SearchTree};

use super::{Node, NodeIndex};

impl Node {
    pub fn expand<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        position: &ChessPosition,
    ) {
        let mut actions = self.actions_mut();

        //Map moves into actions and set initial policy to 1
        position
            .board()
            .map_moves::<_, STM_WHITE, NSTM_WHITE>(|mv| {
                actions.push(Edge::new(NodeIndex::NULL, mv, 1.0))
            });

        //Update the policy to 1/action_count for uniform policy
        let action_count = actions.len() as f32;
        for action in actions.iter_mut() {
            action.update_policy(1.0 / action_count)
        }
    }

    //PUCT formula V + C * P * (N.max(1).sqrt()/n + 1) where N = number of visits to parent node, n = number of visits to a child
    #[inline]
    pub fn select_action<const ROOT: bool>(
        &self,
        tree: &SearchTree,
        node_idx: NodeIndex,
        parent_visits: u32,
        cpuct: f32,
    ) -> usize {
        assert!(self.has_children());

        let explore_value = cpuct * (parent_visits.max(1) as f32).sqrt();
        tree.get_best_action_by_key(node_idx, |action| {
            let visits = action.visits();
            let score = if visits == 0 {
                0.5
            } else {
                action.score() as f32
            };

            score + (explore_value * action.policy() / (visits as f32 + 1.0))
        })
    }
}