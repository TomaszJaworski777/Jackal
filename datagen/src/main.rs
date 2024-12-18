use std::{
    env,
    fs::{File, OpenOptions},
    io::Write,
    sync::atomic::AtomicBool,
    time::Instant,
};

use crossbeam_queue::SegQueue;
use display::Printer;
use policy_gen::PolicyGen;
use jackal::{ChessBoardPacked, PolicyPacked};
use value_gen::ValueGen;

mod display;
mod policy_gen;
mod utils;
mod value_gen;

#[derive(PartialEq, Clone, Copy)]
pub enum DataGenMode {
    Policy,
    Value,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut mode = DataGenMode::Value;
    let mut threads = 1;
    let mut iter_count = 1000;
    let mut target = 1000;
    let mut path = "./value_data.bin";

    let mut cmd = String::new();
    for arg in &args {
        match arg.as_str() {
            "policy" => mode = DataGenMode::Policy,
            "value" => mode = DataGenMode::Value,
            "threads" | "nodes" | "path" | "target" => cmd = arg.clone(),
            _ => {
                match cmd.as_str() {
                    "threads" => threads = arg.parse::<u8>().unwrap_or(1),
                    "nodes" => iter_count = arg.parse::<u32>().unwrap_or(1000),
                    "path" => path = arg.as_str(),
                    "target" => target = arg.parse::<u64>().unwrap_or(1000),
                    _ => continue,
                };
            }
        }
    }

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

    let saved_positions = std::fs::metadata(&path)
        .expect("Cannot get file metadata")
        .len()
        / position_size;
    let target = target * 1_000_000;
    let printer = Printer::new(saved_positions, target, threads, iter_count, mode);

    let save_queue: SegQueue<Vec<u8>> = SegQueue::new();
    let interruption_token = AtomicBool::new(false);

    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                if mode == DataGenMode::Value {
                    ValueGen::start_game_loop(
                        &save_queue,
                        iter_count,
                        &printer,
                        &interruption_token,
                    )
                } else {
                    PolicyGen::start_game_loop(
                        &save_queue,
                        iter_count,
                        &printer,
                        &interruption_token,
                    )
                }
            });
        }

        update_loop(
            &mut file,
            &save_queue,
            &printer,
            target,
            &interruption_token,
        )
    });
}

fn update_loop(
    file: &mut File,
    save_queue: &SegQueue<Vec<u8>>,
    printer: &Printer,
    target: u64,
    interruption_token: &AtomicBool,
) {
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
        } else if interruption_token.load(std::sync::atomic::Ordering::Relaxed) {
            std::process::exit(0)
        }

        if printer.positions() >= target {
            printer.print_report(time);
            interruption_token.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}
