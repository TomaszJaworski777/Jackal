use std::sync::atomic::{AtomicU64, Ordering};

use chess::ZobristKey;

use crate::{AtomicWDLScore, WDLScore};

#[derive(Debug, Default)]
pub struct TableEntry {
    score: AtomicWDLScore,
    hash: AtomicU64,
}

impl Clone for TableEntry {
    fn clone(&self) -> Self {
        Self {
            score: self.score.clone(),
            hash: AtomicU64::new(self.hash.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HashTable(Vec<TableEntry>);
impl HashTable {
    pub fn new(bytes: usize) -> Self {
        let size = bytes / std::mem::size_of::<TableEntry>();

        if size == 0 {
            return Self(Vec::new());
        }

        let entries = unsafe {
            let layout = std::alloc::Layout::array::<TableEntry>(size).unwrap();
            let ptr = std::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            Vec::from_raw_parts(ptr.cast::<TableEntry>(), size, size)
        };

        Self(entries)
    }

    #[inline]
    pub fn bytes(&self) -> usize {
        self.0.len() * std::mem::size_of::<TableEntry>()
    }

    pub fn clear(&self) {
        for entry in &self.0 {
            entry.score.clear();
            entry.hash.store(0, Ordering::Relaxed);
        }
    }

    pub fn get(&self, key: ZobristKey) -> Option<WDLScore> {
        let idx = u64::from(key) % self.0.len() as u64;
        let entry = &self.0[idx as usize];

        if entry.hash.load(Ordering::Relaxed) == u64::from(key) {
            Some(entry.score.get_score())
        } else {
            None
        }
    }

    pub fn push(&self, key: ZobristKey, score: WDLScore) {
        let idx = u64::from(key) % self.0.len() as u64;

        self.0[idx as usize].score.store(score);
        self.0[idx as usize]
            .hash
            .store(u64::from(key), Ordering::Relaxed);
    }
}
