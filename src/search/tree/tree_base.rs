use std::{
    ops::Index,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{search::Score, GameState};

use super::tree_segment::TreeSegment;
use super::{
    node::NodeIndex,
    Edge, Node,
};
use spear::Move;

pub(super) const SEGMENT_COUNT: usize = 2;

pub struct Tree {
    pub(super) segments: [TreeSegment; SEGMENT_COUNT],
    pub(super) root_edge: Edge,
    pub(super) current_segment: AtomicUsize,
    pub(super) tree_size_in_bytes: usize
}

impl Index<NodeIndex> for Tree {
    type Output = Node;

    #[inline]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        assert!(index != NodeIndex::NULL);
        self.segments[index.segment()].get(index)
    }
}

impl Tree {
    pub fn new(size_in_mb: i32) -> Self {
        let bytes = (size_in_mb as usize) * 1024 * 1024;
        let tree_size = bytes as usize / (48 + 12 * 16);
        let segment_size = (tree_size / SEGMENT_COUNT).min(0x7FFFFFFE);
        let segments = [
            TreeSegment::new(segment_size, 0),
            TreeSegment::new(segment_size, 1),
        ];

        let tree = Self {
            segments,
            root_edge: Edge::new(NodeIndex::from_raw(0), Move::NULL, 0.0),
            current_segment: AtomicUsize::new(0),
            tree_size_in_bytes: bytes
        };

        tree
    }

    pub fn resize_tree(&mut self, size_in_mb: i32) {
        *self = Self::new(size_in_mb)
    }

    #[inline]
    pub fn clear(&mut self) {
        for segment in &self.segments {
            segment.clear();
        }

        self.current_segment.store(0, Ordering::Relaxed);
        self.root_edge = Edge::default();

        _ = self.current_segment().add(GameState::Unresolved);
    }

    #[inline]
    pub fn root_index(&self) -> NodeIndex {
        NodeIndex::from_parts(0, self.current_segment.load(Ordering::Relaxed) as u32)
    }

    #[inline]
    pub fn root_edge(&self) -> &Edge {
        &self.root_edge
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
    pub fn add_edge_score(
        &self,
        node_index: NodeIndex,
        action_index: usize,
        score: Score,
    ) {
        self[node_index].actions()[action_index].add_score(score)
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

    pub fn get_pv(&self) -> Vec<Move> {
        let mut result = Vec::new();
        self.get_pv_internal(self.root_index(), &mut result);
        result
    }

    fn get_pv_internal(&self, node_index: NodeIndex, result: &mut Vec<Move>) {
        if !self[node_index].has_children() {
            return;
        }

        //We recursivly descend down the tree picking the best moves and adding them to the result forming pv line
        let best_action = self[node_index].get_best_action(self);
        result.push(self[node_index].actions()[best_action].mv());
        let new_node_index = self[node_index].actions()[best_action].node_index();
        if !new_node_index.is_null() {
            self.get_pv_internal(new_node_index, result)
        }
    }
}
