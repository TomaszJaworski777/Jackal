use std::sync::{
    atomic::{AtomicU16, Ordering},
    RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use spear::Move;

use crate::{
    search::{tree::Edge, Score},
    GameState, Tree,
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
}
