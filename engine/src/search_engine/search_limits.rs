use crate::search_engine::{engine_params::EngineParams, search_stats::ThreadSearchStats};

mod time_manager;

pub use time_manager::TimeManager;

#[derive(Debug, Default)]
pub struct SearchLimits {
    depth: Option<u64>,
    iters: Option<u64>,
    move_time: Option<u128>,
    infinite: bool,
    time_manager: TimeManager
}

impl SearchLimits {
    pub fn set_depth(&mut self, depth: Option<u64>) {
        self.depth = depth
    }

    pub fn set_iters(&mut self, iters: Option<u64>) {
        self.iters = iters
    }
    
    pub fn set_move_time(&mut self, move_time: Option<u128>) {
        self.move_time = move_time
    }

    pub fn set_infinite(&mut self, infinite: bool) {
        self.infinite = infinite
    }

    pub fn is_inifinite(&self) -> bool {
        self.infinite
    }

    pub fn time_manager(&self) -> TimeManager {
        self.time_manager
    }
 
    pub fn is_limit_reached(&self, thread_data: &ThreadSearchStats, iterations: u64, elapsed_ms: u128) -> bool {
        if self.infinite {
            return false;
        }

        if let Some(iters) = self.iters {
            if iterations >= iters {
                return true;
            }
        }

        if let Some(depth) = self.depth {
            if thread_data.avg_depth() >= depth {
                return true;
            }
        }

        if let Some(move_time) = self.move_time {
            if elapsed_ms >= move_time {
                return true;
            }
        }

        false
    }

    pub fn calculate_time_limit(&mut self, time_remaining: Option<u128>, increment: Option<u128>, moves_to_go: Option<u128>, options: &EngineParams, game_ply: u16, phase: f64) {
        self.time_manager.calculate_time_limit(time_remaining, increment, moves_to_go, options, game_ply, phase);
    }
}