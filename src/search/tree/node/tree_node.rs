use std::sync::{
    atomic::{AtomicU16, Ordering},
    RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use crate::search::tree::Edge;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum GameState {
    #[default]
    Unresolved,
    Lost(u8),
    Drawn,
    Won(u8),
}

impl From<GameState> for u16 {
    fn from(value: GameState) -> Self {
        match value {
            GameState::Unresolved => 0,
            GameState::Drawn => 1 << 8,
            GameState::Lost(x) => (2 << 8) ^ u16::from(x),
            GameState::Won(x) => (3 << 8) ^ u16::from(x),
        }
    }
}

impl From<u16> for GameState {
    fn from(value: u16) -> Self {
        let x = value as u8;

        match value >> 8 {
            0 => GameState::Unresolved,
            1 => GameState::Drawn,
            2 => GameState::Lost(x),
            3 => GameState::Won(x),
            _ => unreachable!(),
        }
    }
}

pub struct Node {
    actions: RwLock<Vec<Edge>>,
    state: AtomicU16,
}

impl Node {
    pub fn new(state: GameState) -> Self {
        Self {
            actions: RwLock::new(Vec::new()),
            state: AtomicU16::new(u16::from(state)),
        }
    }

    #[inline]
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
