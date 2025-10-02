use std::{ops::{Index, IndexMut}, sync::atomic::{AtomicU32, Ordering}};

use chess::Move;

mod node;
mod tree_expand;
mod tree_draw;
mod tree_utils;
mod tree_lru;
mod tree_reuse;
mod pv_line;
mod half;

use half::TreeHalf;

pub use node::{Node, GameState, AtomicWDLScore, WDLScore, NodeIndex};
pub use pv_line::PvLine;

use crate::search_engine::{engine_options::EngineOptions, hash_table::HashTable};

#[derive(Debug)]
pub struct Tree {
    halves: [TreeHalf; 2],
    current_half: AtomicU32,
    hash_table: HashTable,
}

impl Clone for Tree {
    fn clone(&self) -> Self {
        Self {
            halves: self.halves.clone(),
            current_half: AtomicU32::from(self.current_half.load(Ordering::Relaxed)),
            hash_table: self.hash_table.clone()
        }
    }
}

impl Index<NodeIndex> for Tree {
    type Output = Node;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.halves[index.half() as usize][index]
    }
}

impl IndexMut<NodeIndex> for Tree {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.halves[index.half() as usize][index]
    }
}

impl Tree {
    pub fn from_bytes(megabytes: usize, options: &EngineOptions) -> Self {
        let bytes = megabytes * 1024 * 1024;
        let hash_bytes = (bytes as f64 * options.hash_size()) as usize;
        let tree_size = Self::bytes_to_size(bytes - hash_bytes);

        let halves = [
            TreeHalf::new(0, tree_size / 2), 
            TreeHalf::new(1, tree_size / 2)
        ];

        halves[0].reserve_nodes(1);

        Self {
            halves,
            current_half: AtomicU32::new(0),
            hash_table: HashTable::new(hash_bytes),
        }
    }

    #[inline]
    pub fn clear(&self) {
        self.halves[0].clear();
        self.halves[1].clear();
        self.hash_table.clear();

        self.current_half.store(0, Ordering::Relaxed);

        self.halves[0].reserve_nodes(1);
        self[self.root_index()].clear(Move::NULL);
    }

    #[inline]
    pub fn current_half_index(&self) -> u32 {
        self.current_half.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn current_half(&self) -> &TreeHalf {
        &self.halves[self.current_half_index() as usize]
    }

    #[inline]
    pub fn root_index(&self) -> NodeIndex {
        NodeIndex::new(self.current_half_index(), 0)
    }

    #[inline]
    pub fn root_node(&self) -> &Node {
        &self[self.root_index()]
    }
    
    #[inline]
    pub fn max_size(&self) -> usize {
        self.halves[0].max_size() + self.halves[1].max_size()
    }

    #[inline]
    pub fn current_size(&self) -> usize {
        self.halves[0].current_size() + self.halves[1].current_size()
    }

    #[inline]
    pub fn hash_table(&self) -> &HashTable {
        &self.hash_table
    }

    #[inline]
    pub fn add_visit(&self, node_idx: NodeIndex, score: WDLScore) {
        self[node_idx].add_visit(score)
    }

    #[inline]
    pub fn set_state(&self, node_idx: NodeIndex, state: GameState) {
        self[node_idx].set_state(state)
    }

    #[inline]
    pub fn inc_threads(&self, node_idx: NodeIndex, value: u8) -> u8 {
        self[node_idx].inc_threads(value)
    }

    #[inline]
    pub fn dec_threads(&self, node_idx: NodeIndex, value: u8) -> u8 {
        self[node_idx].dec_threads(value)
    }
}
