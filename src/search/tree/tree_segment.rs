use std::sync::atomic::{AtomicU32, Ordering};

use super::{GameState, Node, NodeIndex};

pub struct TreeSegment {
    nodes: Vec<Node>,
    length: AtomicU32,
    segment: u32,
}

impl TreeSegment {
    pub fn new(size: usize, segment_index: u32) -> Self {
        let mut segment = Self {
            nodes: Vec::with_capacity(size),
            length: AtomicU32::new(0),
            segment: segment_index,
        };

        for _ in 0..size {
            segment.nodes.push(Node::new(super::GameState::Unresolved));
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

    pub fn add(&self, state: GameState, path: &str) -> NodeIndex {
        let new_index = NodeIndex::from_parts(self.length.load(Ordering::Relaxed), self.segment);
        assert!(new_index.index() < self.size() as u32, "{path}");
        self.get(new_index).replace(state);
        self.length.fetch_add(1, Ordering::Relaxed);
        new_index
    }

    pub fn clear_references(&self, target_segment: u32) {
        Self::clear_references_internal(&self.nodes, target_segment);
    }

    fn clear_references_internal(nodes: &[Node], target_segment: u32) {
        for node in nodes {
            for action in &*node.actions() {
                if action.node_index().segment() == target_segment as usize {
                    action.set_index(NodeIndex::NULL);
                }
            }
        }
    }
}
