use std::sync::{
        atomic::{AtomicU16, Ordering},
        RwLock, RwLockReadGuard, RwLockWriteGuard,
    };

use super::Edge;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum GameState {
    #[default]
    Unresolved,
    Lost(u8),
    Drew,
    Won(u8),
}

impl From<GameState> for u16 {
    fn from(value: GameState) -> Self {
        match value {
            GameState::Unresolved => 0,
            GameState::Drew => 1 << 8,
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
            1 => GameState::Drew,
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

    pub fn replace(&self, state: GameState) {
        *self.actions_mut() = Vec::new();
        self.state.store(u16::from(state), Ordering::Relaxed)
    }

    pub fn state(&self) -> GameState {
        GameState::from(self.state.load(Ordering::Relaxed))
    }

    pub fn set_state(&self, state: GameState) {
        self.state.store(u16::from(state), Ordering::Relaxed)
    }

    pub fn actions(&self) -> RwLockReadGuard<Vec<Edge>> {
        self.actions.read().unwrap()
    }

    pub fn actions_mut(&self) -> RwLockWriteGuard<Vec<Edge>> {
        self.actions.write().unwrap()
    }

    pub fn has_children(&self) -> bool {
        self.actions().len() > 0
    }

    pub fn is_termial(&self) -> bool {
        self.state() != GameState::Unresolved
    }
}
