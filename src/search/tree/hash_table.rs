use std::sync::atomic::{AtomicU16, Ordering};

use crate::search::{eval_score::AtomicScore, Score};

pub struct HashTableEntry {
    key: AtomicU16,
    score: AtomicScore,
}

impl HashTableEntry {
    pub fn new() -> Self {
        Self {
            key: AtomicU16::new(0),
            score: AtomicScore::default(),
        }
    }

    pub fn replace(&self, key: u64, score: Score) {
        self.key.store(key as u16, Ordering::Relaxed);
        self.score.store(score);
    }
}

pub struct HashTable {
    entries: Vec<HashTableEntry>,
}

impl HashTable {
    pub fn new(size_in_bytes: usize) -> Self {
        let size = size_in_bytes / 8;
        let mut entries = Vec::with_capacity(size);

        for _ in 0..size {
            entries.push(HashTableEntry::new());
        }

        Self { entries }
    }

    pub fn resize(&mut self, size_in_bytes: usize) {
        *self = HashTable::new(size_in_bytes)
    }

    pub fn clear(&mut self) {
        let size = self.entries.len();
        let mut entries = Vec::with_capacity(size);

        for _ in 0..size {
            entries.push(HashTableEntry::new());
        }

        *self = Self { entries }
    }

    pub fn probe(&self, key: u64) -> Option<Score> {
        let idx = (u128::from(key).wrapping_mul(self.entries.len() as u128) >> 64) as usize;
        let entry = &self.entries[idx];
        if entry.key.load(Ordering::Relaxed) != key as u16 {
            return None;
        }

        Some(entry.score.load())
    }

    pub fn store(&self, key: u64, score: Score) {
        let idx = (u128::from(key).wrapping_mul(self.entries.len() as u128) >> 64) as usize;
        self.entries[idx].replace(key, score);
    }
}
