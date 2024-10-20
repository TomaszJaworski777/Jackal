use std::sync::{
    atomic::{AtomicU16, Ordering},
    RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use spear::{ChessPosition, Move, Side};

use crate::{
    search::{tree::Edge, NodeIndex, Score}, EngineOptions, GameState, PolicyNetwork, Tree
};

pub struct Node {
    actions: RwLock<Vec<Edge>>,
    state: AtomicU16,
}

impl Default for Node {
    fn default() -> Self {
        Self::new(GameState::Unresolved)
    }
}

impl Node {
    pub fn new(state: GameState) -> Self {
        Self {
            actions: RwLock::new(Vec::new()),
            state: AtomicU16::new(u16::from(state)),
        }
    }

    pub fn clear(&self) {
        self.replace(GameState::Unresolved);
    }

    #[inline]
    pub fn replace(&self, state: GameState) {
        *self.actions_mut() = Vec::new();
        self.state.store(u16::from(state), Ordering::Relaxed)
    }

    #[inline]
    pub fn state(&self) -> GameState {
        GameState::from(self.state.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn set_state(&self, state: GameState) {
        self.state.store(u16::from(state), Ordering::Relaxed)
    }

    #[inline]
    pub fn actions(&self) -> RwLockReadGuard<Vec<Edge>> {
        self.actions.read().unwrap()
    }

    #[inline]
    pub fn actions_mut(&self) -> RwLockWriteGuard<Vec<Edge>> {
        self.actions.write().unwrap()
    }

    #[inline]
    pub fn has_children(&self) -> bool {
        self.actions().len() > 0
    }

    #[inline]
    pub fn is_termial(&self) -> bool {
        self.state() != GameState::Unresolved
    }

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

    pub fn get_best_action_by_key<F: FnMut(&Edge) -> f32>(&self, mut method: F) -> usize {
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

    pub fn expand<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        position: &ChessPosition,
        options: &EngineOptions
    ) {
        let mut actions = self.actions_mut();

        let mut inputs: Vec<usize> = Vec::with_capacity(32);
        PolicyNetwork::map_policy_inputs::<_, STM_WHITE, NSTM_WHITE>(position.board(), |idx| {
            inputs.push(idx)
        });

        let vertical_flip = if position.board().side_to_move() == Side::WHITE {
            0
        } else {
            56
        };

        let pst = if ROOT {
            options.root_pst()
        } else {
            1.0
        };

        let mut max = f32::NEG_INFINITY;
        let mut total = 0.0;

        const MULTIPLIER: f32 = 1000.0;
        position
            .board()
            .map_moves::<_, STM_WHITE, NSTM_WHITE>(|mv| {
                let policy = PolicyNetwork.forward(&inputs, mv, vertical_flip);
                actions.push(Edge::new(
                    NodeIndex::from_raw((policy * MULTIPLIER) as u32),
                    mv,
                    0.0,
                ));
                max = max.max(policy);
            });

        for action in actions.iter_mut() {
            let policy = action.node_index().get_raw() as f32 / MULTIPLIER;
            let policy = ((policy - max) / pst).exp();
            total += policy;
            action.set_node_index(NodeIndex::from_raw((policy * MULTIPLIER) as u32));
        }

        let is_single_action = actions.len() == 1;
        for action in actions.iter_mut() {
            let policy = action.node_index().get_raw() as f32 / MULTIPLIER;
            let policy = if is_single_action {
                1.0
            } else {
                policy / total
            };
            action.update_policy(policy);
            action.set_node_index(NodeIndex::NULL);
        }
    }

    pub fn recalculate_policy<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        position: &ChessPosition,
        options: &EngineOptions
    ) {
        let mut actions = self.actions_mut();

        let mut inputs: Vec<usize> = Vec::with_capacity(32);
        PolicyNetwork::map_policy_inputs::<_, STM_WHITE, NSTM_WHITE>(position.board(), |idx| {
            inputs.push(idx)
        });

        let vertical_flip = if position.board().side_to_move() == Side::WHITE {
            0
        } else {
            56
        };

        let pst = if ROOT {
            options.root_pst()
        } else {
            1.0
        };

        let mut policies = Vec::new();
        let mut max: f32 = f32::NEG_INFINITY;
        let mut total = 0.0;

        for action in actions.iter_mut() {
            let policy = PolicyNetwork.forward(&inputs, action.mv(), vertical_flip);
            policies.push(policy);
            max = max.max(policy);
        }
        
        let is_single_action = actions.len() == 1;
        for policy in policies.iter_mut() {
            if is_single_action {
                *policy = 1.0;
                total = 1.0;
            } else {
                *policy = ((*policy - max) / pst).exp();
                total += *policy;
            }
        }

        for (i, action) in actions.iter_mut().enumerate() {
            action.update_policy(policies[i] / total);
        }
    }
}