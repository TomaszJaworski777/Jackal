use std::{
    ops::Index,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::tree_segment::TreeSegment;
use super::{
    node::{GameState, NodeIndex},
    Edge, Node,
};
use spear::{ChessBoard, Move, Side, ZobristKey};

const SEGMENT_COUNT: usize = 4;

pub struct SearchTree {
    segments: [TreeSegment; SEGMENT_COUNT],
    root_edge: Edge,
    current_segment: AtomicUsize,
}

impl SearchTree {
    pub fn new(size_in_mb: i64) -> Self {
        let bytes = size_in_mb * 1024 * 1024;
        let tree_size =
            bytes as usize / (std::mem::size_of::<Node>() + 8 * std::mem::size_of::<Edge>());
        let segment_size = (tree_size / SEGMENT_COUNT).min(0x3FFFFFFE);
        let segments = [
            TreeSegment::new(segment_size, 0),
            TreeSegment::new(segment_size, 1),
            TreeSegment::new(segment_size, 2),
            TreeSegment::new(segment_size, 3),
        ];

        let tree = Self {
            segments,
            root_edge: Edge::new(NodeIndex::from_raw(0), Move::NULL, 0.0),
            current_segment: AtomicUsize::new(0),
        };

        tree.init_root();
        tree
    }

    pub fn resize_tree(&mut self, size_in_mb: i64) {
        *self = Self::new(size_in_mb)
    }

    pub fn reuse_tree(&mut self, previous_board: &ChessBoard, current_board: &ChessBoard) {
        let current_key = current_board.get_key();
        if previous_board.get_key() == current_key {
            return;
        }

        let node_index = if previous_board.side_to_move() == Side::WHITE {
            self.find_position::<true, false>(previous_board, current_key, self.root_index(), 2)
        } else {
            self.find_position::<false, true>(previous_board, current_key, self.root_index(), 2)
        };

        if let Some((edge_idx, action_idx)) = node_index {
            self.root_edge = self.get_edge_clone(edge_idx, action_idx);
        } else {
            self.clear();
        }
    }

    fn find_position<const STM_WHITE: bool, const NSTM_WHITE: bool>(&self, previous_board: &ChessBoard, key: ZobristKey, node_index: NodeIndex, depth: u8) -> Option<(NodeIndex, usize)> {
        let actions = &*self[node_index].actions();
        for (index, action) in actions.iter().enumerate() {
            let child_index = action.node_index();
            if child_index == NodeIndex::NULL {
                continue;
            }

            let mut board_clone = previous_board.clone();
            board_clone.make_move::<STM_WHITE, NSTM_WHITE>(action.mv());
            if board_clone.get_key() == key {
                return Some((node_index, index));
            }

            if depth - 1 > 0 {
                let result = self.find_position::<NSTM_WHITE, STM_WHITE >(&board_clone, key, child_index, depth - 1);
                if result.is_some() {
                    return result;
                }
            }
        }

        None
    }

    #[inline]
    pub fn clear(&mut self) {
        for segment in &self.segments {
            segment.clear();
        }

        self.current_segment.store(0, Ordering::Relaxed);
        self.root_edge = Edge::new(NodeIndex::from_raw(0), Move::NULL, 0.0);
        self.init_root();
    }

    #[inline]
    fn init_root(&self) {
        let root_index = self.spawn_node(GameState::Unresolved);
        self.root_edge.set_index(root_index);
    }

    #[inline]
    pub fn root_index(&self) -> NodeIndex {
        self.root_edge.node_index()
    }

    #[inline]
    pub fn root_edge(&self) -> Edge {
        self.root_edge.clone()
    }

    #[inline]
    pub fn current_segment(&self) -> &TreeSegment {
        &self.segments[self.current_segment.load(Ordering::Relaxed)]
    }

    #[inline]
    pub fn total_usage(&self) -> f32 {
        let mut total = 0.0;

        for idx in 0..SEGMENT_COUNT {
            total += self.segments[idx].len() as f32 / self.segments[idx].size() as f32
        }

        total / SEGMENT_COUNT as f32
    }

    #[inline]
    pub fn get_edge_clone(&self, node_index: NodeIndex, action_index: usize) -> Edge {
        self[node_index].actions()[action_index].clone()
    }

    #[inline]
    pub fn change_edge_node_index(
        &self,
        edge_node_index: NodeIndex,
        action_index: usize,
        new_node_index: NodeIndex,
    ) {
        self[edge_node_index].actions()[action_index].set_index(new_node_index)
    }

    #[inline]
    pub fn add_edge_score<const ROOT: bool>(
        &self,
        node_index: NodeIndex,
        action_index: usize,
        score: f32,
    ) {
        if ROOT {
            self.root_edge.add_score(score)
        } else {
            self[node_index].actions()[action_index].add_score(score)
        }
    }

    pub fn mark_as_used<const ROOT: bool>(
        &self,
        index: NodeIndex,
        edge_node_index: NodeIndex,
        action_index: usize,
    ) -> NodeIndex {
        if index.segment() == self.current_segment.load(Ordering::Relaxed) {
            return index;
        }

        if self.current_segment().is_full() {
            self.advance_segments();
        }

        let old_node = &self[index];
        let new_index = self.current_segment().add(old_node.state());
        assert!(new_index != NodeIndex::NULL);

        if index != new_index {
            let old_actions = &mut *old_node.actions_mut();
            let new_actions = &mut *self[new_index].actions_mut();
            std::mem::swap(old_actions, new_actions);
        }

        if ROOT {
            self.root_edge.set_index(new_index);
        } else {
            self.change_edge_node_index(edge_node_index, action_index, new_index);
        }

        new_index
    }

    #[inline]
    pub fn spawn_node(&self, state: GameState) -> NodeIndex {
        if self.current_segment().is_full() {
            self.advance_segments();
        }

        self.current_segment().add(state)
    }

    fn advance_segments(&self) {
        let current_segment_index = self.current_segment.load(Ordering::Relaxed);
        let new_segment_index = (current_segment_index + 1) % SEGMENT_COUNT;

        for i in 0..SEGMENT_COUNT {
            self.segments[i].clear_references(new_segment_index as u32);
        }

        self.current_segment
            .store(new_segment_index, Ordering::Relaxed);
        self.segments[new_segment_index].clear();
    }

    pub fn backpropagate_mates(&self, parent_node_index: NodeIndex, child_state: GameState) {
        match child_state {
            GameState::Lost(x) => self[parent_node_index].set_state(GameState::Won(x + 1)),
            GameState::Won(x) => {
                //To backpropagate won state we need to check, if all states are won (forced check mate)
                //and if we are sure that its forced, then we select the longest line and we pray that enemy will miss it
                let mut proven_loss = true;
                let mut longest_win_length = x;
                for action in self[parent_node_index].actions().iter() {
                    if action.node_index().is_null() {
                        proven_loss = false;
                        break;
                    } else if let GameState::Won(x) = self[action.node_index()].state() {
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
            _ => (),
        }
    }

    pub fn get_best_move(&self, node_index: NodeIndex) -> (Move, f64) {
        //Extracts the best move from all possible root moves
        let action_index = self.get_best_action(node_index);

        //If no action was selected then return null move
        if action_index == usize::MAX {
            return (Move::NULL, self.root_edge.score());
        }

        let edge_clone = self.get_edge_clone(node_index, action_index);
        (edge_clone.mv(), edge_clone.score())
    }

    pub fn get_best_action(&self, node_index: NodeIndex) -> usize {
        self.get_best_action_by_key(node_index, |action| {
            if action.visits() == 0 {
                f32::NEG_INFINITY
            } else if !action.node_index().is_null() {
                match self[action.node_index()].state() {
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
        mut method: F,
    ) -> usize {
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

    fn get_pv_internal(&self, node_index: NodeIndex, result: &mut Vec<Move>) {
        if !self[node_index].has_children() {
            return;
        }

        //We recursivly desent down the tree picking the best moves and adding them to the result forming pv line
        let best_action = self.get_best_action(node_index);
        result.push(self[node_index].actions()[best_action].mv());
        let new_node_index = self[node_index].actions()[best_action].node_index();
        if !new_node_index.is_null() {
            self.get_pv_internal(new_node_index, result)
        }
    }

    pub fn draw_tree_from_root(&self, depth: u32) {
        self.root_edge()
            .print::<true>(0.5, 0.5, self[self.root_index()].state(), true);
        self.draw_tree(self.root_index(), depth)
    }

    pub fn draw_tree(&self, node_index: NodeIndex, depth: u32) {
        self.draw_tree_internal(node_index, depth - 1, &String::new(), false)
    }

    fn draw_tree_internal(
        &self,
        node_index: NodeIndex,
        depth: u32,
        prefix: &String,
        flip_score: bool,
    ) {
        if self.total_usage() == 0.0 {
            return;
        }

        let max_policy = self[node_index]
            .actions()
            .iter()
            .max_by(|&a, &b| {
                a.policy()
                    .partial_cmp(&b.policy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&Edge::new(NodeIndex::NULL, Move::NULL, 0.0))
            .policy();

        let min_policy = self[node_index]
            .actions()
            .iter()
            .min_by(|&a, &b| {
                a.policy()
                    .partial_cmp(&b.policy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&Edge::new(NodeIndex::NULL, Move::NULL, 0.0))
            .policy();

        let actions_len = self[node_index].actions().len();
        for (index, action) in self[node_index].actions().iter().enumerate() {
            let is_last = index == actions_len - 1;
            let state = if action.node_index().is_null() {
                GameState::Unresolved
            } else {
                self[action.node_index()].state()
            };
            print!("{}{} ", prefix, if is_last { "└─>" } else { "├─>" });
            action.print::<false>(min_policy, max_policy, state, flip_score);
            if !action.node_index().is_null()
                && self[action.node_index()].has_children()
                && depth > 0
            {
                let prefix_add = if is_last { "    " } else { "│   " };
                self.draw_tree_internal(
                    action.node_index(),
                    depth - 1,
                    &format!("{}{}", prefix, prefix_add),
                    !flip_score,
                )
            }
        }
    }
}

impl Index<NodeIndex> for SearchTree {
    type Output = Node;

    #[inline]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        assert!(index != NodeIndex::NULL);
        self.segments[index.segment()].get(index)
    }
}
