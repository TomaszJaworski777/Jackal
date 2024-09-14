use core::f32;
use std::{
    ops::{Index, IndexMut},
    sync::atomic::{AtomicI32, Ordering},
};

use spear::Move;

use super::{node::GameState, Edge, Node};

pub struct SearchTree {
    values: Vec<Node>,
    root_edge: Edge,
    last_index: AtomicI32,
}

impl SearchTree {
    pub fn new() -> Self {
        let mut tree = Self {
            values: Vec::new(),
            root_edge: Edge::new(0, Move::NULL, 0.0),
            last_index: AtomicI32::new(0),
        };

        for _ in 0..100_000_000 {
            tree.values.push(Node::new(GameState::Unresolved))
        }

        tree.init_root();
        tree
    }

    #[inline]
    pub fn clear(&mut self) {
        self.last_index.store(0, Ordering::Relaxed);
        self.root_edge = Edge::new(0, Move::NULL, 0.0);
        self.init_root();
    }

    #[inline]
    fn init_root(&self) {
        let root_index = self.spawn_node(GameState::Unresolved);
        self.root_edge.set_index(root_index);
    }

    #[inline]
    pub fn root_index(&self) -> i32 {
        self.root_edge.index()
    }

    #[inline]
    pub fn root_edge(&self) -> Edge {
        self.root_edge.clone()
    }

    #[inline]
    pub fn get_edge_clone(&self, node_index: i32, action_index: usize) -> Edge {
        self[node_index].actions()[action_index].clone()
    }

    #[inline]
    pub fn change_edge_node_index(&self, edge_node_index: i32, action_index: usize, new_node_index: i32) {
        self[edge_node_index].actions()[action_index].set_index(new_node_index)
    }

    #[inline]
    pub fn add_edge_score<const ROOT: bool>(
        &self,
        node_index: i32,
        action_index: usize,
        score: f32,
    ) {
        if ROOT {
            self.root_edge.add_score(score)
        } else {
            self[node_index].actions()[action_index].add_score(score)
        }
    }

    #[inline]
    pub fn spawn_node(&self, state: GameState) -> i32 {
        let new_node_index = self.last_index.load(Ordering::Relaxed);
        self[new_node_index].replace(state);
        self.last_index.fetch_add(1, Ordering::Relaxed);
        new_node_index as i32
    }

    pub fn backpropagate_mates(&self, parent_node_index: i32, child_state: GameState) {
        match child_state {
            GameState::Lost(x) => self[parent_node_index].set_state(GameState::Won(x + 1)),
            GameState::Won(x) => {
                //To backpropagate won state we need to check, if all states are won (forced check mate)
                //and if we are sure that its forced, then we select the longest line and we pray that enemy will miss it
                let mut proven_loss = true;
                let mut longest_win_length = x;
                for action in self[parent_node_index].actions().iter() {
                    if action.index() == -1 {
                        proven_loss = false;
                        break;
                    } else if let GameState::Won(x) = self[action.index()].state() {
                        longest_win_length = x.max(longest_win_length);
                    } else {
                        proven_loss = false;
                        break;
                    }
                }

                if proven_loss {
                    self[parent_node_index].set_state(GameState::Lost(longest_win_length + 1));
                }
            }
            _ => return,
        }
    }

    pub fn get_best_move(&self, node_index: i32) -> (Move, f64) {
        //Extracts the best move from all possible root moves
        let action_index = self.get_best_action(node_index, |action| {
            if action.visits() == 0 {
                return f32::MIN;
            }

            action.score() as f32
        });

        //If no action was selected then return null move
        if action_index == usize::MAX {
            return (Move::NULL, self.root_edge.score());
        }

        let edge_clone = self.get_edge_clone(node_index, action_index);
        (edge_clone.mv(), edge_clone.score())
    }

    pub fn get_best_action<F: FnMut(&Edge) -> f32>(&self, node_index: i32, mut method: F) -> usize {
        let mut best_action_index = usize::MAX;
        let mut best_score = f32::MIN;

        for (index, action) in self[node_index].actions().iter().enumerate() {
            let score = method(action);
            if score >= best_score {
                best_action_index = index;
                best_score = score;
            }
        }

        best_action_index
    }

    #[inline]
    pub fn get_pv(&self) -> Vec<Move> {
        let mut result = Vec::new();
        self.get_pv_internal(self.root_index(), &mut result);
        result
    }

    fn get_pv_internal(&self, node_index: i32, result: &mut Vec<Move>) {

        if !self[node_index].has_children() || self[node_index].is_termial() {
            return;
        }

        //We recursivly desent down the tree picking the best moves and adding them to the result forming pv line
        let best_action = self.get_best_action(node_index, | x | {
            x.score() as f32
        });
        result.push(self[node_index].actions()[best_action].mv());
        let new_node_index = self[node_index].actions()[best_action].index();
        if new_node_index == -1 {
            return;
        } else {
            self.get_pv_internal(new_node_index, result)
        }
    }
}

impl Index<i32> for SearchTree {
    type Output = Node;

    #[inline]
    fn index(&self, index: i32) -> &Self::Output {
        &self.values[index as usize]
    }
}

impl IndexMut<i32> for SearchTree {
    #[inline]
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.values[index as usize]
    }
}
