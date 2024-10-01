use std::sync::atomic::{AtomicUsize, Ordering};

use crate::GameState;

use super::{Node, NodeIndex};

pub struct TreeSegment {
    nodes: Vec<Node>,
    length: AtomicUsize,
    segment: u32,
}

impl TreeSegment {
    pub fn new(size: usize, segment_index: u32) -> Self {
        let mut segment = Self {
            nodes: Vec::with_capacity(size),
            length: AtomicUsize::new(0),
            segment: segment_index,
        };

        for _ in 0..size {
            segment.nodes.push(Node::new(GameState::Unresolved));
        }

        segment
    }

    pub fn clear(&self) {
        self.length.store(0, Ordering::Relaxed);
    }

    pub fn len(&self) -> usize {
        self.length.load(Ordering::Relaxed) as usize
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_full(&self) -> bool {
        self.len() >= self.nodes.len()
    }

    pub fn get(&self, index: NodeIndex) -> &Node {
        &self.nodes[index.index() as usize]
    }

    pub fn add(&self, state: GameState) -> Option<NodeIndex> {
        let new_index = self.length.fetch_add(1, Ordering::Relaxed);

        if new_index >= self.nodes.len() {
            return None;
        }
        
        self.nodes[new_index].replace(state);
        Some(NodeIndex::from_parts(new_index as u32, self.segment))
    }

    pub fn clear_references(&self, target_segment: u32) {
        Self::clear_references_internal(&self.nodes, target_segment);
    }

    fn clear_references_internal(nodes: &[Node], target_segment: u32) {
        for node in nodes {
            for action in &*node.actions() {
                if action.node_index().segment() == target_segment as usize {
                    action.set_node_index(NodeIndex::NULL);
                }
            }
        }
    }
}
