use core::f32;
use std::hash::{DefaultHasher, Hash, Hasher};
use flurry::{Guard, HashMap};
use spear::{ChessPosition, Move};

use super::{
    node::{GameState, NodeIndex},
    Edge, Node,
};

pub struct SearchTree {
    values: HashMap<NodeIndex, Node>,
    root_edge: Edge,
}

impl Default for SearchTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchTree {
    pub fn new() -> Self {
        Self {
            values: HashMap::with_capacity(10_000_000),
            root_edge: Edge::new(NodeIndex::new(0), Move::NULL, 0.0),
        }
    }

    #[inline]
    pub fn clear(&mut self, position: &ChessPosition) {
        self.root_edge = Edge::new(NodeIndex::new(0), Move::NULL, 0.0);
        let guard = self.values.guard();
        self.values.clear(&guard);
        self.init_root(position, &guard);
    }

    #[inline]
    fn init_root(&self, position: &ChessPosition, guard: &Guard) {
        let root_index = self.spawn_node(position, GameState::Unresolved, guard);
        self.root_edge.set_index(root_index);
    }

    #[inline]
    pub fn root_index(&self) -> NodeIndex {
        self.root_edge.index()
    }

    #[inline]
    pub fn root_edge(&self) -> Edge {
        self.root_edge.clone()
    }

    #[inline]
    pub fn get_guard(&self) -> Guard {
        self.values.guard()
    }

    #[inline]
    pub fn get<'b>(&'b self, index: NodeIndex, guard: &'b Guard) -> &'b Node {
        self.values.get(&index, guard).unwrap()
    }

    #[inline]
    pub fn get_edge_clone(
        &self,
        node_index: NodeIndex,
        action_index: usize,
        guard: &Guard,
    ) -> Edge {
        self.get(node_index, guard).actions()[action_index].clone()
    }

    #[inline]
    pub fn change_edge_node_index(
        &self,
        edge_node_index: NodeIndex,
        action_index: usize,
        new_node_index: NodeIndex,
        guard: &Guard,
    ) {
        self.get(edge_node_index, guard).actions()[action_index].set_index(new_node_index)
    }

    #[inline]
    pub fn add_edge_score<const ROOT: bool>(
        &self,
        node_index: NodeIndex,
        action_index: usize,
        score: f32,
        guard: &Guard,
    ) {
        if ROOT {
            self.root_edge.add_score(score)
        } else {
            self.get(node_index, guard).actions()[action_index].add_score(score)
        }
    }

    #[inline]
    pub fn spawn_node(
        &self,
        position: &ChessPosition,
        state: GameState,
        guard: &Guard,
    ) -> NodeIndex {
        let mut hasher = DefaultHasher::default();
        position.hash(&mut hasher);
        let new_node_index = NodeIndex::new(hasher.finish());
        let _result = self
            .values
            .try_insert(new_node_index, Node::new(state), guard);
        new_node_index
    }

    pub fn backpropagate_mates(
        &self,
        parent_node_index: NodeIndex,
        child_state: GameState,
        guard: &Guard,
    ) {
        match child_state {
            GameState::Lost(x) => self
                .get(parent_node_index, guard)
                .set_state(GameState::Won(x + 1)),
            GameState::Won(x) => {
                //To backpropagate won state we need to check, if all states are won (forced check mate)
                //and if we are sure that its forced, then we select the longest line and we pray that enemy will miss it
                let mut proven_loss = true;
                let mut longest_win_length = x;
                for action in self.get(parent_node_index, guard).actions().iter() {
                    if action.index().is_null() {
                        proven_loss = false;
                        break;
                    } else if let GameState::Won(x) = self.get(action.index(), guard).state() {
                        longest_win_length = x.max(longest_win_length);
                    } else {
                        proven_loss = false;
                        break;
                    }
                }

                if proven_loss {
                    self.get(parent_node_index, guard)
                        .set_state(GameState::Lost(longest_win_length + 1));
                }
            }
            _ => (),
        }
    }

    pub fn get_best_move(&self, node_index: NodeIndex, guard: &Guard) -> (Move, f64) {
        //Extracts the best move from all possible root moves
        let action_index = self.get_best_action(node_index, guard);

        //If no action was selected then return null move
        if action_index == usize::MAX {
            return (Move::NULL, self.root_edge.score());
        }

        let edge_clone = self.get_edge_clone(node_index, action_index, guard);
        (edge_clone.mv(), edge_clone.score())
    }

    pub fn get_best_action(&self, node_index: NodeIndex, guard: &Guard) -> usize {
        self.get_best_action_by_key(node_index, guard, |action| {
            if action.visits() == 0 {
                f32::NEG_INFINITY
            } else if !action.index().is_null() {
                match self.get(action.index(), guard).state() {
                    GameState::Lost(n) => 1.0 + f32::from(n),
                    GameState::Won(n) => f32::from(n) - 256.0,
                    GameState::Drawn => 0.5,
                    GameState::Unresolved => action.score() as f32,
                }
            } else {
                action.score() as f32
            }
        })
    }

    pub fn get_best_action_by_key<F: FnMut(&Edge) -> f32>(
        &self,
        node_index: NodeIndex,
        guard: &Guard,
        mut method: F,
    ) -> usize {
        let mut best_action_index = usize::MAX;
        let mut best_score = f32::MIN;

        for (index, action) in self.get(node_index, guard).actions().iter().enumerate() {
            let score = method(action);
            if score >= best_score {
                best_action_index = index;
                best_score = score;
            }
        }

        best_action_index
    }

    #[inline]
    pub fn get_pv(&self, guard: &Guard) -> Vec<Move> {
        let mut result = Vec::new();
        self.get_pv_internal(self.root_index(), &mut result, guard);
        result
    }

    fn get_pv_internal(&self, node_index: NodeIndex, result: &mut Vec<Move>, guard: &Guard) {
        if !self.get(node_index, guard).has_children() {
            return;
        }

        //We recursivly desent down the tree picking the best moves and adding them to the result forming pv line
        let best_action = self.get_best_action(node_index, guard);
        result.push(self.get(node_index, guard).actions()[best_action].mv());
        let new_node_index = self.get(node_index, guard).actions()[best_action].index();
        if !new_node_index.is_null() {
            self.get_pv_internal(new_node_index, result, guard)
        }
    }

    pub fn draw_tree_from_root(&self, depth: u32, guard: &Guard) {
        self.root_edge()
            .print::<true>(0.5, 0.5, self.get(self.root_index(), guard).state(), true);
        self.draw_tree(self.root_index(), depth, guard)
    }

    pub fn draw_tree(&self, node_index: NodeIndex, depth: u32, guard: &Guard) {
        self.draw_tree_internal(node_index, depth - 1, &String::new(), false, guard)
    }

    fn draw_tree_internal(
        &self,
        node_index: NodeIndex,
        depth: u32,
        prefix: &String,
        flip_score: bool,
        guard: &Guard,
    ) {
        let max_policy = self
            .get(node_index, guard)
            .actions()
            .iter()
            .max_by(|&a, &b| {
                a.policy()
                    .partial_cmp(&b.policy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap()
            .policy();

        let min_policy = self
            .get(node_index, guard)
            .actions()
            .iter()
            .min_by(|&a, &b| {
                a.policy()
                    .partial_cmp(&b.policy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap()
            .policy();

        let actions_len = self.get(node_index, guard).actions().len();
        for (index, action) in self.get(node_index, guard).actions().iter().enumerate() {
            let is_last = index == actions_len - 1;
            let state = if action.index().is_null() {
                GameState::Unresolved
            } else {
                self.get(action.index(), guard).state()
            };
            print!("{}{} ", prefix, if is_last { "└─>" } else { "├─>" });
            action.print::<false>(min_policy, max_policy, state, flip_score);
            if !action.index().is_null()
                && self.get(action.index(), guard).has_children()
                && depth > 0
            {
                let prefix_add = if is_last { "    " } else { "│   " };
                self.draw_tree_internal(
                    action.index(),
                    depth - 1,
                    &format!("{}{}", prefix, prefix_add),
                    !flip_score,
                    guard,
                )
            }
        }
    }
}
