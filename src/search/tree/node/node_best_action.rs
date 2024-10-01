use spear::Move;

use crate::{search::{tree::Edge, Score}, GameState, Tree};

use super::Node;

impl Node {
    pub fn get_best_move(&self, tree: &Tree) -> (Move, Score) {
        //Extracts the best move from all possible root moves
        let action_index = self.get_best_action(tree);

        //If no action was selected then return null move
        if action_index == usize::MAX {
            return (Move::NULL, Score::DRAW);
        }

        let edge_clone = &self.actions()[action_index];
        (edge_clone.mv(), edge_clone.score())
    }

    pub fn get_best_action(&self, tree: &Tree) -> usize {
        self.get_best_action_by_key(|action| {
            if action.visits() == 0 {
                f32::NEG_INFINITY
            } else if !action.node_index().is_null() {
                match tree[action.node_index()].state() {
                    GameState::Lost(n) => 1.0 + f32::from(n),
                    GameState::Won(n) => f32::from(n) - 256.0,
                    GameState::Drawn => 0.5,
                    GameState::Unresolved => f32::from(action.score()),
                }
            } else {
                f32::from(action.score())
            }
        })
    }

    pub fn get_best_action_by_key<F: FnMut(&Edge) -> f32>(
        &self,
        mut method: F,
    ) -> usize {
        let mut best_action_index = usize::MAX;
        let mut best_score = f32::MIN;

        for (index, action) in self.actions().iter().enumerate() {
            let score = method(action);
            if score >= best_score {
                best_action_index = index;
                best_score = score;
            }
        }

        best_action_index
    }
}