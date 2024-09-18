use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use crate::{spear::StringUtils, utils::clear_terminal_screen};

pub struct Printer {
    positions: AtomicU64,
    positions_since_last_raport: AtomicU64,
    full_timer: Instant,
    target: u64,
    threads: u8
}

impl Printer {
    pub fn new(position_count: u64, target: u64, threads: u8) -> Self {
        Self {
            positions: AtomicU64::new(position_count),
            positions_since_last_raport: AtomicU64::new(0),
            full_timer: Instant::now(),
            target,
            threads
        }
    }

    pub fn positions(&self) -> u64 {
        self.positions.load(Ordering::Relaxed)
    }

    pub fn add_position(&self, amount: u64) {
        self.positions.fetch_add(amount, Ordering::Relaxed);
        self.positions_since_last_raport
            .fetch_add(amount, Ordering::Relaxed);
    }

    pub fn print_report(&self, time_since_last_raport_in_ms: u128) {
        clear_terminal_screen();
        let positions_since_last_raport = self.positions_since_last_raport.load(Ordering::Relaxed);
        let positions = self.positions.load(Ordering::Relaxed);
        let time = self.full_timer.elapsed().as_secs();
        let hours = time / 3600;
        let mins = (time - (hours * 3600)) / 60;
        let secs = time - (hours * 3600) - (mins * 60);

        let positions_per_second = positions_since_last_raport as f32 * 1000.0 / time_since_last_raport_in_ms as f32;

        let e_time = ((self.target - positions.min(self.target)) as f32 / positions_per_second.max(1.0)) as u64;
        let e_hours = e_time / 3600;
        let e_mins = (e_time - (e_hours * 3600)) / 60;
        let e_secs = e_time - (e_hours * 3600) - (e_mins * 60);

        println!("Generating data in progress...");
        println!("{}", Self::get_loading_bar(positions, self.target, 50));
        println!("Positions:                {}/{} ({:.1} per second)", StringUtils::large_number_to_string(positions as u128), StringUtils::large_number_to_string(self.target as u128), positions_per_second);
        println!("Threads:                  {} + 1", self.threads);
        println!("Time passed:              {}h{}m{}s", hours, mins, secs);
        println!("Estimated time remaining: {}h{}m{}s", e_hours, e_mins, e_secs);

        self.positions_since_last_raport.store(0, Ordering::Relaxed);
    }

    fn get_loading_bar(points: u64, total: u64, length: usize) -> String {
        let mut result = String::new();
        let filled_spots = ((points as f64 / total as f64) * length as f64) as usize;
        result.push_str("[");

        for i in 0..length {
            let character = if i < filled_spots {
                "#"
            } else {
                "-"
            };

            result.push_str(character);
        }

        result.push_str("] ");
        result.push_str(format!("{}%", ((points as f64 / total as f64) * 100.0) as usize).as_str());
        result
    }
}
