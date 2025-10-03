use std::{ops::{Index, IndexMut}, sync::atomic::{AtomicUsize, Ordering}};

use crate::{Node, NodeIndex};

#[derive(Debug)]
pub struct TreeHalf {
    nodes: Vec<Node>,
    idx: AtomicUsize,
    half_index: u64
}

impl Clone for TreeHalf {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            idx: AtomicUsize::new(self.idx.load(Ordering::Relaxed)),
            half_index: self.half_index,
        }
    }
}

impl Index<NodeIndex> for TreeHalf {
    type Output = Node;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.nodes[index.idx() as usize]
    }
}

impl IndexMut<NodeIndex> for TreeHalf {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.nodes[index.idx() as usize]
    }
}

impl TreeHalf {
    pub fn new(index: u64, size: usize) -> Self {
        let nodes = unsafe {
            let layout = std::alloc::Layout::array::<Node>(size).unwrap();
            let ptr = std::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            Vec::from_raw_parts(ptr.cast::<Node>(), size, size)
        };

        Self { 
            nodes, 
            idx: AtomicUsize::new(0),
            half_index: index
        }
    }

    #[inline]
    pub fn half_index(&self) -> u64 {
        self.half_index
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.current_size() >= self.max_size()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.current_size() == 0
    }

    #[inline]
    pub fn current_size(&self) -> usize {
        self.idx.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn max_size(&self) -> usize {
        self.nodes.len()
    }
    
    #[inline]
    pub fn reserve_nodes(&self, count: usize) -> Option<NodeIndex> {
        let current_idx = self.idx.fetch_add(count, Ordering::Relaxed);

        if current_idx + count >= self.nodes.len() {
            return None;
        }

        Some(NodeIndex::new(self.half_index, current_idx as u64))
    }

    pub fn clear(&self) {
        self.idx.store(0, Ordering::Relaxed);
    }

    #[inline]
    pub fn clear_references(&self) {
        for node in &self.nodes {
            if node.children_index().half() == self.half_index {
                continue;
            }

            node.clear_children();
        }
    }
}