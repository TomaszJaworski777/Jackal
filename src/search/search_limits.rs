use crate::options::EngineOptions;

use super::SearchStats;

#[derive(Default)]
pub struct SearchLimits {
    time_remaining: Option<u64>,
    increment: Option<u64>,
    moves_to_go: Option<u32>,
    move_time: Option<u64>,
    max_depth: Option<u32>,
    max_iters: Option<u32>,
    infinite: bool,
}
impl SearchLimits {
    pub fn new() -> Self {
        Self {
            time_remaining: None,
            increment: None,
            moves_to_go: None,
            move_time: None,
            max_depth: None,
            max_iters: None,
            infinite: false,
        }
    }

    pub fn add_time_remaining(&mut self, time_remaining: u64) {
        self.time_remaining = Some(time_remaining);
    }

    pub fn add_increment(&mut self, increment: u64) {
        self.increment = Some(increment);
    }

    pub fn add_moves_to_go(&mut self, moves_to_go: u32) {
        self.moves_to_go = Some(moves_to_go);
    }

    pub fn add_move_time(&mut self, move_time: u64) {
        self.move_time = Some(move_time);
    }

    pub fn add_depth(&mut self, depth: u32) {
        self.max_depth = Some(depth);
    }

    pub fn add_iters(&mut self, iters: u32) {
        self.max_iters = Some(iters);
    }

    pub fn go_infinite(&mut self) {
        self.infinite = true;
    }

    pub fn is_limit_reached(&self, search_stats: &SearchStats, options: &EngineOptions) -> bool {
        if self.infinite {
            return false;
        }

        if let Some(max_depth) = self.max_depth {
            if search_stats.avg_depth() >= max_depth {
                return true;
            }
        }

        if let Some(max_iters) = self.max_iters {
            if search_stats.iters() >= max_iters {
                return true;
            }
        }

        if let Some(time) = self.time_remaining {
            if search_stats.time_passed() + options.move_overhead() as u64
                >= Self::search_time(time, self.increment, self.moves_to_go)
            {
                return true;
            }
        }

        if let Some(time) = self.move_time {
            if search_stats.time_passed() >= time {
                return true;
            }
        }

        false
    }

    fn search_time(time: u64, increment: Option<u64>, moves_to_go: Option<u32>) -> u64 {
        let inc = increment.unwrap_or_default();

        if let Some(mtg) = moves_to_go {
            return (time + inc) / mtg as u64;
        }

        time / 20 + inc / 2
    }
}
