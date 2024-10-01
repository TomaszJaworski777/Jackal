use std::sync::{
    atomic::{AtomicU16, Ordering},
    RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use crate::{search::tree::Edge, GameState};

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
}
