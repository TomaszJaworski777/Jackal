use std::{
    ops::Index,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{search::SearchHelpers, GameState};

use super::tree_segment::TreeSegment;
use super::{
    node::NodeIndex,
    Edge, Node,
};
use spear::{ChessBoard, ChessPosition, Move, Side};

const SEGMENT_COUNT: usize = 2;

pub struct Tree {
    segments: [TreeSegment; SEGMENT_COUNT],
    root_edge: Edge,
    current_segment: AtomicUsize,
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
    pub fn new(size_in_mb: i64) -> Self {
        let bytes = size_in_mb * 1024 * 1024;
        let tree_size =
            bytes as usize / (std::mem::size_of::<Node>() + 8 * std::mem::size_of::<Edge>());
        let segment_size = (tree_size / SEGMENT_COUNT).min(0x7FFFFFFE);
        let segments = [
            TreeSegment::new(segment_size, 0),
            TreeSegment::new(segment_size, 1),
        ];

        let tree = Self {
            segments,
            root_edge: Edge::new(NodeIndex::from_raw(0), Move::NULL, 0.0),
            current_segment: AtomicUsize::new(0),
        };

        tree
    }

    pub fn resize_tree(&mut self, size_in_mb: i64) {
        *self = Self::new(size_in_mb)
    }

    pub fn reuse_tree(&mut self, previous_board: &ChessBoard, current_board: &ChessBoard) {
        if self.total_usage() == 0.0 {
            _ = self.current_segment().add(GameState::Unresolved);
            return;
        }

        if previous_board == current_board {
            return;
        }

        let (node_index, edge) = if previous_board.side_to_move() == Side::WHITE {
            self.recurse_find::<_, true, false>(self.root_index(), previous_board, self.root_edge().clone(), 2, &|board, _| board == current_board)
        } else {
            self.recurse_find::<_, false, true>(self.root_index(), previous_board, self.root_edge().clone(), 2, &|board, _| board == current_board)
        };

        if !node_index.is_null() && self[node_index].has_children() {
            self[self.root_index()].replace(GameState::Unresolved);
            self.copy_node(node_index, self.root_index());
            self.root_edge = edge;
        } else {
            self.clear();
            _ = self.current_segment().add(GameState::Unresolved);
        }
    }

    fn recurse_find<F: Fn(&ChessBoard, NodeIndex) -> bool, const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        start: NodeIndex,
        board: &ChessBoard,
        edge: Edge,
        depth: u8,
        method: &F
    ) -> (NodeIndex, Edge) {
        if method(board, start) {
            return (start, edge);
        }

        if start.is_null() || depth == 0 {
            return (NodeIndex::NULL, Edge::default());
        }

        let node = &self[start];

        for action in node.actions().iter() {
            let child_index = action.node_index();
            let mut child_board = board.clone();

            child_board.make_move::<STM_WHITE, NSTM_WHITE>(action.mv());

            let (idx, edge) =
                self.recurse_find::<F, NSTM_WHITE, STM_WHITE>(child_index, &child_board, action.clone(), depth - 1, method);

            if !idx.is_null() {
                return (idx, edge);
            }
        }

        (NodeIndex::NULL, Edge::default())
    }

    #[inline]
    pub fn clear(&mut self) {
        for segment in &self.segments {
            segment.clear();
        }

        self.current_segment.store(0, Ordering::Relaxed);
        self.root_edge = Edge::default();
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

    pub fn get_node_index<const STM_WHITE: bool, const NSTM_WHITE: bool>(&self, position: &ChessPosition, child_index: NodeIndex, edge_index: NodeIndex, action_index: usize) -> Option<NodeIndex> {

        //When there is no node assigned to the selected move edge, we spawn new node
        //and return its index
        if child_index.is_null() {

            //We spawn a new node and update the corresponding edge. If the segment returned None,
            //then it means segment is full, we return that instantly and process it in the search
            let state = SearchHelpers::get_position_state::<STM_WHITE, NSTM_WHITE>(position);
            let new_index = self.current_segment().add(state)?;

            self[edge_index].actions()[action_index].set_node_index(new_index);

            Some(new_index)

        //When there is a node assigned to the selected move edge, but the assigned
        //node is in old tree segment, we want to copy it to the new tree segment
        } else if child_index.segment() != self.current_segment.load(Ordering::Relaxed) {

            //We get new index from the segment. If the index is None, then segment is
            //full. When that happens we return it instantly and process it in the search
            let new_index = self.current_segment().add(GameState::Unresolved)?;

            //Next, we copy the actions from the old node to the new one and
            self.copy_node(child_index, new_index);
            self[edge_index].actions()[action_index].set_node_index(new_index);

            Some(new_index)

        //When nthere is a node assigned to the selected move edge and it's located
        //in corrected segment, we can just return the index without changes
        } else {
            Some(child_index)
        }
    }

    pub fn advance_segments(&self) {
        let old_root_index = self.root_index();

        let current_segment_index = self.current_segment.load(Ordering::Relaxed);
        let new_segment_index = (current_segment_index + 1) % SEGMENT_COUNT;

        for i in 0..SEGMENT_COUNT {
            if i != new_segment_index {
                self.segments[i].clear_references(new_segment_index as u32);
            }
        }

        self.current_segment
            .store(new_segment_index, Ordering::Relaxed);
        self.segments[new_segment_index].clear();

        let new_root_index = self.segments[new_segment_index].add(GameState::Unresolved).unwrap();
        self[new_root_index].replace(GameState::Unresolved);

        self.copy_node(old_root_index, new_root_index);
    }

    fn copy_node(&self, a: NodeIndex, b: NodeIndex) {
        if a == b {
            return;
        }

        let a_actions = &mut *self[a].actions_mut();
        let b_actions = &mut *self[b].actions_mut();
        
        self[b].set_state(self[a].state());

        if a_actions.is_empty() {
            return;
        }

        std::mem::swap(a_actions, b_actions);
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
}
