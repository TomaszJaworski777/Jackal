use crate::{search_engine::{engine_options::EngineOptions, SearchStats}};

mod time_manager;

pub use time_manager::TimeManager;

#[derive(Debug, Default)]
pub struct SearchLimits {
    depth: Option<u64>,
    iters: Option<u64>,
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
    
    pub fn set_infinite(&mut self, infinite: bool) {
        self.infinite = infinite
    }

    pub fn is_inifinite(&self) -> bool {
        self.infinite
    }

    pub fn time_manager(&self) -> TimeManager {
        self.time_manager
    }
 
    pub fn is_limit_reached(&self, search_stats: &SearchStats) -> bool {
        if self.infinite {
            return false;
        }

        if let Some(iters) = self.iters {
            if search_stats.iterations() >= iters {
                return true;
            }
        }

        if let Some(depth) = self.depth {
            if search_stats.avg_depth() >= depth {
                return true;
            }
        }

        false
    }

    pub fn set_time(&mut self, time: u128) {
        self.time_manager.set_time(time);
    }

    pub fn calculate_time_limit(&mut self, time_remaining: Option<u128>, increment: Option<u128>, moves_to_go: Option<u128>, options: &EngineOptions, game_ply: u16, phase: f64) {
        self.time_manager.calculate_time_limit(time_remaining, increment, moves_to_go, options, game_ply, phase);
    }
}