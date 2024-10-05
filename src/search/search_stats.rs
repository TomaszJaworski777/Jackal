use std::{
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    time::Instant,
};

pub struct SearchStats {
    timer: Instant,
    total_depth: AtomicU64,
    max_depth: AtomicU32,
    iters: AtomicU64,
    time_passed: AtomicU64,
}

impl Default for SearchStats {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchStats {
    pub fn new() -> Self {
        Self {
            timer: Instant::now(),
            total_depth: AtomicU64::new(0),
            max_depth: AtomicU32::new(0),
            iters: AtomicU64::new(0),
            time_passed: AtomicU64::new(0),
        }
    }

    pub fn time_passed(&self) -> u64 {
        self.time_passed.load(Ordering::Relaxed)
    }

    pub fn avg_depth(&self) -> u32 {
        (self.total_depth.load(Ordering::Relaxed) / self.iters() as u64) as u32
    }

    pub fn max_depth(&self) -> u32 {
        self.max_depth.load(Ordering::Relaxed)
    }

    pub fn iters(&self) -> u32 {
        self.iters.load(Ordering::Relaxed) as u32
    }

    pub fn add_iteration(&self, depth: u32) {
        self.iters.fetch_add(1, Ordering::Relaxed);
        self.total_depth.fetch_add(depth as u64, Ordering::Relaxed);
        self.max_depth
            .store(self.max_depth().max(depth), Ordering::Relaxed);
    }

    pub fn update_time_passed(&self) {
        self.time_passed
            .store(self.timer.elapsed().as_millis() as u64, Ordering::Relaxed)
    }
}
