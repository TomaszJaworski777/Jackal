use std::ops::{Index, IndexMut};

use spear::Move;

use super::{Edge, Node};

pub struct SearchTree {
    values: Vec<Node>,
    root_edge: Edge
}

impl SearchTree {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            root_edge: Edge::new(0, Move::NULL, 0.0)
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.root_edge = Edge::new(0, Move::NULL, 0.0);
    }

    pub fn root_index(&self) -> i32 {
        self.root_edge.index()
    }
}

impl Index<i32> for SearchTree {
    type Output = Node;

    fn index(&self, index: i32) -> &Self::Output {
        &self.values[index as usize]
    }
}

impl IndexMut<i32> for SearchTree {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.values[index as usize]
    }
}