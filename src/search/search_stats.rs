use std::{sync::atomic::{AtomicU32, Ordering}, time::Instant};

pub struct SearchStats {
    timer: Instant,
    total_depth: AtomicU32,
    max_depth: AtomicU32,
    iters: AtomicU32,
}
impl SearchStats {
    pub fn new() -> Self {
        Self {
            timer: Instant::now(),
            total_depth: AtomicU32::new(0),
            max_depth: AtomicU32::new(0),
            iters: AtomicU32::new(0),
        }
    }

    pub fn time_elapsed_milis(&self) -> u128 {
        self.timer.elapsed().as_millis()
    }

    pub fn time_elapsed_secs(&self) -> f64 {
        self.timer.elapsed().as_secs_f64()
    }

    pub fn avg_depth(&self) -> u32 {
        self.total_depth.load(Ordering::Relaxed) / self.iters()
    }

    pub fn max_depth(&self) -> u32 {
        self.max_depth.load(Ordering::Relaxed)
    }

    pub fn iters(&self) -> u32 {
        self.iters.load(Ordering::Relaxed)
    }

    pub fn add_iteration(&self, depth: u32) {
        self.iters.fetch_add(1, Ordering::Relaxed);
        self.total_depth.fetch_add(depth, Ordering::Relaxed);
        self.max_depth.store(self.max_depth().max(depth), Ordering::Relaxed);
    }
}
