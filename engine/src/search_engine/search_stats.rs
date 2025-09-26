use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

pub struct SearchStats {
    iterations: AtomicU64,
    cumulative_depth: AtomicU64,
    max_depth: AtomicU64,
    timer: Instant,
}

impl SearchStats {
    pub fn new(_threads: usize) -> Self {
        SearchStats {
            iterations: AtomicU64::new(0),
            cumulative_depth: AtomicU64::new(0),
            max_depth: AtomicU64::new(0),
            timer: Instant::now(),
        }
    }

    pub fn iterations(&self) -> u64 {
        self.iterations.load(Ordering::Relaxed)
    }

    pub fn avg_depth(&self) -> u64 {
        self.cumulative_depth.load(Ordering::Relaxed) / self.iterations().max(1)
    }

    pub fn max_depth(&self) -> u64 {
        self.max_depth.load(Ordering::Relaxed)
    }

    pub fn cumulative_depth(&self) -> u64 {
        self.cumulative_depth.load(Ordering::Relaxed)
    }

    pub fn time_passesd_ms(&self) -> u128 {
        self.timer.elapsed().as_millis()
    }

    pub fn add_iteration(&self, depth: u64) {
        self.iterations.fetch_add(1, Ordering::Relaxed);
        self.cumulative_depth
            .fetch_add(depth, Ordering::Relaxed);
        self.max_depth.fetch_max(depth, Ordering::Relaxed);
    }
}
