use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

pub struct SearchStats {
    threads: Vec<ThreadSearchStats>,
    timer: Instant,
}

#[repr(align(64))]
pub struct ThreadSearchStats {
    iterations: AtomicU64,
    cumulative_depth: AtomicU64,
    max_depth: AtomicU64,
}

impl ThreadSearchStats {
    pub fn new() -> Self {
        Self {
            iterations: AtomicU64::new(0),
            cumulative_depth: AtomicU64::new(0),
            max_depth: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    pub fn iterations(&self) -> u64 {
        self.iterations.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn avg_depth(&self) -> u64 {
        self.cumulative_depth.load(Ordering::Relaxed) / self.iterations().max(1)
    }

    #[inline(always)]
    pub fn max_depth(&self) -> u64 {
        self.max_depth.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn cumulative_depth(&self) -> u64 {
        self.cumulative_depth.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn add_batch(&self, accumulator: &SearchStatsAccumulator) {
        self.iterations.fetch_add(accumulator.iterations(), Ordering::Relaxed);
        self.cumulative_depth.fetch_add(accumulator.cumulative_depth(), Ordering::Relaxed);
        self.max_depth.fetch_max(accumulator.max_depth(), Ordering::Relaxed);
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SearchStatsAccumulator {
    iterations: u64,
    cumulative_depth: u64,
    max_depth: u64,
}

impl SearchStatsAccumulator {
    #[inline(always)]
    pub fn iterations(&self) -> u64 {
        self.iterations
    }

    #[inline(always)]
    pub fn cumulative_depth(&self) -> u64 {
        self.cumulative_depth
    }

    #[inline(always)]
    pub fn avg_depth(&self) -> u64 {
        self.cumulative_depth() / self.iterations().max(1)
    }

    #[inline(always)]
    pub fn max_depth(&self) -> u64 {
        self.max_depth
    }

    #[inline(always)]
    pub fn add_iteration(&mut self, depth: u64) {
        self.iterations += 1;
        self.cumulative_depth += depth;
        self.max_depth = self.max_depth.max(depth);
    }
}

impl SearchStats {
    pub fn new(threads: usize) -> Self {
        let threads = (0..threads).map(|_| ThreadSearchStats::new()).collect();

        SearchStats {
            threads,
            timer: Instant::now(),
        }
    }

    #[inline(always)]
    pub fn thread_stats(&self, thread_idx: usize) -> &ThreadSearchStats {
        &self.threads[thread_idx]
    }

    pub fn aggregate(&self) -> SearchStatsAccumulator {
        let mut result = SearchStatsAccumulator::default();
        for thread in self.threads.iter() {
            result.iterations += thread.iterations.load(Ordering::Relaxed);
            result.cumulative_depth += thread.cumulative_depth.load(Ordering::Relaxed);
            result.max_depth = thread.max_depth.load(Ordering::Relaxed).max(result.max_depth);
        }

        result
    }

    #[inline(always)]
    pub fn elapsed_ms(&self) -> u128 {
        self.timer.elapsed().as_millis()
    }
}
