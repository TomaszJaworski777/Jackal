use std::sync::atomic::{AtomicI16, AtomicI32, AtomicU32, Ordering};

use spear::Move;

pub struct Edge {
    node_index: AtomicI32,
    mv: Move,
    policy: AtomicI16,
    visits: AtomicU32,
    score: AtomicU32,
}

impl Clone for Edge {
    fn clone(&self) -> Self {
        Self {
            node_index: AtomicI32::new(self.index()),
            mv: self.mv(),
            policy: AtomicI16::new(self.policy.load(Ordering::Relaxed)),
            visits: AtomicU32::new(self.visits()),
            score: AtomicU32::new(self.score.load(Ordering::Relaxed)),
        }
    }
}

impl Edge {
    pub fn new(node_index: i32, mv: Move, policy: f32) -> Self {
        Self {
            node_index: AtomicI32::new(node_index),
            mv,
            policy: AtomicI16::new((policy * f32::from(i16::MAX)) as i16),
            visits: AtomicU32::new(0),
            score: AtomicU32::new(0),
        }
    }

    pub fn index(&self) -> i32 {
        self.node_index.load(Ordering::Relaxed)
    }

    pub fn set_index(&self, index: i32) {
        self.node_index.store(index, Ordering::Relaxed)
    }

    pub fn mv(&self) -> Move {
        self.mv
    }

    pub fn policy(&self) -> f32 {
        f32::from(self.policy.load(Ordering::Relaxed)) / f32::from(i16::MAX)
    }

    pub fn visits(&self) -> u32 {
        self.visits.load(Ordering::Relaxed)
    }

    pub fn score(&self) -> f64 {
        f64::from(self.score.load(Ordering::Relaxed)) / f64::from(u32::MAX)
    }

    pub fn add_score(&self, score: f32) {
        let score = f64::from(score);
        let previous_visits = self.visits.fetch_add(1, Ordering::Relaxed) as f64;
        let new_score = (self.score() * previous_visits + score) / (previous_visits + 1.0);
        self.score
            .store((new_score * f64::from(u32::MAX)) as u32, Ordering::Relaxed)
    }

    pub fn update_policy(&self, new_policy: f32) {
        self.policy
            .store((new_policy * f32::from(i16::MAX)) as i16, Ordering::Relaxed)
    }
}
