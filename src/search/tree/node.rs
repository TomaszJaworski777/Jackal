use super::{Edge, SearchTree};

#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    Unresolved,
    Lose(u8),
    Draw,
    Win(u8),
}

#[derive(Clone, PartialEq)]
pub struct Node {
    actions: Vec<Edge>,
    state: GameState,
    edge: (i32, u8)
}

impl Node {
    pub fn new( state: GameState, edge_index: i32, action_index: u8 ) -> Self {
        Self {
            actions: Vec::new(),
            state,
            edge: (edge_index, action_index)
        }
    }

    pub fn get_edge(&self) -> (i32, u8) {
        self.edge
    }

    pub fn clear_edge(&mut self) {
        self.edge = (-1, 0)
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn set_state(&mut self, state: GameState) {
        self.state = state
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.state = GameState::Unresolved;
    }

    //expand
}