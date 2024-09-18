use std::{
    fs::{File, OpenOptions}, io::Write, time::Instant
};

use crossbeam_queue::SegQueue;

use crate::spear::{ChessBoardPacked, PolicyPacked};

use super::{printer::Printer, value_gen::ValueGen};

#[derive(PartialEq)]
pub enum DataGenMode {
    Policy,
    Value,
}

pub struct DataGen;
impl DataGen {
    pub fn start_gen(threads: u8, iter_count: u32, path: &str, target: u64, mode: DataGenMode) {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .expect("Cannot open file");

        let position_size = if mode == DataGenMode::Value {
            std::mem::size_of::<ChessBoardPacked>() as u64
        } else {
            std::mem::size_of::<PolicyPacked>() as u64
        };

        let saved_positions = std::fs::metadata(&path).expect("Cannot get file metadata").len() / position_size;
        let target = target * 1_000_000;
        let printer = Printer::new(saved_positions, target, threads);

        let save_queue: SegQueue<Vec<u8>> = SegQueue::new();

        std::thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| {
                    if mode == DataGenMode::Value {
                        ValueGen::start_game_loop(&save_queue, iter_count, &printer)
                    } else {
                        //policy data gen loop
                    }
                });
            }

            Self::update_loop(&mut file, &save_queue, &printer, target)
        });
    }

    fn update_loop(file: &mut File, save_queue: &SegQueue<Vec<u8>>, printer: &Printer, target: u64) {
        let mut timer = Instant::now();
        loop {
            let time = timer.elapsed().as_millis();
            if time > 1000 {
                printer.print_report(time);
                timer = Instant::now()
            }

            if save_queue.len() > 0 {
                let buffer = save_queue.pop().expect("Cannot obtain save buffer");
                file.write_all(&buffer)
                    .expect("Error while writing to file")
            }

            if printer.positions() >= target {
                printer.print_report(time);
                std::process::exit(0)
            }
        }
    }
}
