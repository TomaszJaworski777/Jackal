use std::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, AtomicU8, Ordering};

use chess::Move;

use crate::search_engine::tree::node::{game_state::AtomicGameState, lock::{IndexLock, IndexLockGuard}, wdl_score::SCORE_SCALE};

mod game_state;
mod wdl_score;
mod node_index;
mod lock;

pub use game_state::GameState;
pub use wdl_score::{WDLScore, AtomicWDLScore};
pub use node_index::NodeIndex;

#[derive(Debug)]
pub struct Node {
    mv: AtomicU16,
    visit_count: AtomicU32,
    cumulative_score: AtomicWDLScore,
    squared_score: AtomicU64,
    children_start_index: IndexLock,
    children_count: AtomicU8,
    policy: AtomicU16,
    state: AtomicGameState,
    threads: AtomicU8,
    gini_impurity: AtomicU16
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            mv: AtomicU16::new(self.mv.load(Ordering::Relaxed)),
            visit_count: AtomicU32::new(self.visit_count.load(Ordering::Relaxed)),
            cumulative_score: self.cumulative_score.clone(),
            squared_score: AtomicU64::new(self.squared_score.load(Ordering::Relaxed)),
            children_start_index: self.children_start_index.clone(),
            children_count: AtomicU8::new(self.children_count.load(Ordering::Relaxed)),
            state: self.state.clone(),
            policy: AtomicU16::new(self.policy.load(Ordering::Relaxed)),
            threads: AtomicU8::new(self.threads.load(Ordering::Relaxed)),
            gini_impurity: AtomicU16::new(self.gini_impurity.load(Ordering::Relaxed))
        }
    }
}

impl Node {
    pub fn new() -> Self {
        Self {
            mv: AtomicU16::new(0),
            visit_count: AtomicU32::new(0),
            cumulative_score: AtomicWDLScore::default(),
            squared_score: AtomicU64::new(0),
            children_start_index: IndexLock::new(NodeIndex::NULL),
            children_count: AtomicU8::new(0),
            state: AtomicGameState::new(GameState::Ongoing),
            policy: AtomicU16::new(0),
            threads: AtomicU8::new(0),
            gini_impurity: AtomicU16::new(0)
        }
    }

    #[inline]
    pub fn set_to(&self, node: &Node) {
        self.mv.store(u16::from(node.mv()), Ordering::Relaxed);
        self.visit_count.store(node.visits(), Ordering::Relaxed);
        self.cumulative_score.store(node.cumulative_score.get_score());
        self.squared_score.store(node.squared_score.load(Ordering::Relaxed), Ordering::Relaxed);
        self.state.set(node.state());
        self.policy.store(node.policy.load(Ordering::Relaxed), Ordering::Relaxed);
        self.threads.store(node.threads(), Ordering::Relaxed);
        self.gini_impurity.store(node.gini_impurity.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    #[inline]
    pub fn clear(&self, mv: Move) {
        self.mv.store(u16::from(mv), Ordering::Relaxed);
        self.visit_count.store(0, Ordering::Relaxed);
        self.cumulative_score.clear();
        self.squared_score.store(0, Ordering::Relaxed);
        self.state.set(GameState::Ongoing);
        self.policy.store(0, Ordering::Relaxed);
        self.threads.store(0, Ordering::Relaxed);
        self.gini_impurity.store(0, Ordering::Relaxed);
        self.clear_children();
    }

    pub fn clear_children(&self) { 
        self.children_index_mut().store(NodeIndex::NULL);
        self.children_count.store(0, Ordering::Relaxed);
    }

    #[inline]
    pub fn mv(&self) -> Move {
        Move::from(self.mv.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn visits(&self) -> u32 {
        self.visit_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn score(&self) -> WDLScore {
        self.cumulative_score.get_score_with_visits(self.visits())
    }

    #[inline]
    pub fn squared_score(&self) -> f64 {
        ((self.squared_score.load(Ordering::Relaxed) as f64) / f64::from(SCORE_SCALE)) / f64::from(self.visits())
    }

    #[inline]
    pub fn children_index(&self) -> NodeIndex {
        self.children_start_index.read()
    }

    #[inline]
    pub fn children_index_mut(&self) -> IndexLockGuard<'_> {
        self.children_start_index.write()
    }

    #[inline]
    pub fn children_count(&self) -> usize {
        self.children_count.load(Ordering::Relaxed) as usize
    }

    #[inline]
    pub fn state(&self) -> GameState {
        self.state.get()
    }

    #[inline]
    pub fn policy(&self) -> f64 {
        self.policy.load(Ordering::Relaxed) as f64 / f64::from(u16::MAX)
    }

    #[inline]
    pub fn threads(&self) -> u8 {
        self.threads.load(Ordering::Relaxed)
    }
    
    #[inline]
    pub fn gini_impurity(&self) -> f64 {
        f64::from(self.gini_impurity.load(Ordering::Relaxed)) / f64::from(u16::MAX)
    }

    #[inline]
    pub fn is_terminal(&self) -> bool {
        self.state() != GameState::Ongoing
    }

    #[inline]
    pub fn set_state(&self, state: GameState) {
        self.state.set(state)
    }

    #[inline]
    pub fn set_policy(&self, policy: f64) {
        self.policy.store((policy * f64::from(u16::MAX)) as u16, Ordering::Relaxed)
    }

    #[inline]
    pub fn inc_threads(&self, value: u8) -> u8 {
        self.threads.fetch_add(value, Ordering::Relaxed)
    }

    #[inline]
    pub fn dec_threads(&self, value: u8) -> u8 {
        self.threads.fetch_sub(value, Ordering::Relaxed)
    }

    #[inline]
    pub fn set_gini_impurity(&self, gini_impurity: f64) {
        self.gini_impurity.store((gini_impurity * f64::from(u16::MAX)) as u16, Ordering::Relaxed)
    }

    #[inline]
    pub fn add_visit(&self, score: WDLScore) {
        self.visit_count.fetch_add(1, Ordering::Relaxed) as f64;
        self.cumulative_score.add(score);
        
        let score = score.single() as f64;
        self.squared_score.fetch_add((score.powi(2) * f64::from(SCORE_SCALE)) as u64, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_children_count(&self, chilren_count: usize) {
        self.children_count.store(chilren_count as u8, Ordering::Relaxed);
    }

    pub fn map_children<F: FnMut(NodeIndex)>(&self, mut func: F) {
        let children_idx = self.children_index();

        for child_idx in 0..self.children_count() {
            func(children_idx + child_idx)
        }
    }
}
