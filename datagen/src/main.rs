use std::{env, fs::{File, OpenOptions}, io::{BufRead, Write}, sync::atomic::{AtomicU64, Ordering}, time::Duration};

use chess::{ChessBoard, ChessPosition, FEN};
use crossbeam::queue::SegQueue;
use engine::{SearchEngine, SearchLimits};
use utils::{clear_terminal_screen, create_loading_bar, WHITE};
use rand::Rng;

use crate::game::play_game;

mod game;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut threads = 1usize;
    let mut target_positions = 100_000_000usize;
    #[allow(unused_mut)]
    let mut output_path = "./value_data.bin".to_string();

    #[cfg(feature = "policy_datagen")] {
        output_path = "./policy_data.bin".to_string();
    }

    for i in 0..args.len() {
        match args[i].as_str() {
            "--threads" => {
                if let Some(v) = args.get(i + 1) {
                    threads = v.parse().unwrap_or(1);
                }
            }
            "--target" => {
                if let Some(v) = args.get(i + 1) {
                    target_positions = v.parse().unwrap_or(100) * 1_000_000;
                }
            }
            _ => {}
        }
    }

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(output_path)
        .expect("Cannot open file");

    let openings = std::io::BufReader::new(File::open("./resources/books/UHO_Lichess_4852_v1.epd")
                                    .expect("Book does not exist!"))
                                    .lines().map(|line| line.unwrap())
                                    .collect::<Vec<String>>();

    let mut limits = SearchLimits::default();
    limits.set_iters(Some(99999));

    let datagen_stats = DatagenStats::new();

    let save_queue: SegQueue<Vec<u8>> = SegQueue::new();

    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                let mut engine = SearchEngine::new();
                _ = engine.set_option("Contempt", "250");
                _ = engine.set_option("DrawScore", "50");
                _ = engine.set_option("PolicySac", "0");
                _ = engine.set_option("kld_min", "0.000006");

                let mut rng = rand::rng();
                
                loop {
                    let fen = FEN::from(openings[rng.random_range(0..openings.len())].clone());
                    let mut new_position = ChessPosition::from(ChessBoard::from(&fen));
                    
                    let mut avg_iters = 0u64;
                    let game = play_game(&mut engine, &mut new_position, &limits, &mut avg_iters);
                    datagen_stats.add_game(game.moves.len() as u64, (game.result * 2.0) as i32 - 1, avg_iters);

                    let mut buffer = Vec::new();
                    let _ = game.serialise_into_buffer(&mut buffer);

                    save_queue.push(buffer);
                };
            });
        }
 
        let mut last_update_positions = 0;

        loop {
            clear_terminal_screen();

            #[cfg(feature = "policy_datagen")] {
                println!("Generating policy data...\n");
            }

            #[cfg(feature = "value_datagen")] {
                println!("Generating value data...\n");
            }

            let positions = datagen_stats.positions();
            let speed = positions - last_update_positions;
            last_update_positions = positions;

            let seconds_left = (target_positions as u128 - positions as u128) / speed.max(1) as u128;

            println!("Progress:   {}", create_loading_bar(40, positions as f32 / target_positions as f32, WHITE, WHITE));
            println!("Threads:    {}", threads);
            println!("Avg. Nodes: {:.1}", datagen_stats.avg_iterations());
            println!("Target:     {}\n", utils::number_to_string(target_positions as u128));

            println!("Positions:  {}", utils::number_to_string(positions as u128));
            println!("Games:      {}", utils::number_to_string(datagen_stats.games() as u128));
            println!("WDL:        [W: {}, D: {}, L: {}]\n", 
                utils::number_to_string(datagen_stats.wins() as u128), 
                utils::number_to_string(datagen_stats.draws() as u128), 
                utils::number_to_string(datagen_stats.loses() as u128)
            );

            println!("Speed:      {}N/s", utils::number_to_string(speed as u128));
            println!("ETA:        {}", utils::time_to_string(seconds_left * 1000));

            while save_queue.len() > 0 {
                let buffer = save_queue.pop().expect("Cannot obtain save buffer"); 
                file.write_all(&buffer).expect("Error while writing to file")
            }

            if positions >= target_positions as u64 {
                std::process::exit(0);
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

struct DatagenStats {
    positions: AtomicU64,
    games: AtomicU64,
    white_wins: AtomicU64,
    draws: AtomicU64,
    iterations: AtomicU64
}

impl DatagenStats {
    fn new() -> Self {
        Self { 
            positions: AtomicU64::new(0), 
            games: AtomicU64::new(0), 
            white_wins: AtomicU64::new(0), 
            draws: AtomicU64::new(0),
            iterations: AtomicU64::new(0),
        }
    }

    fn positions(&self) -> u64 {
        self.positions.load(Ordering::Relaxed)
    } 

    fn games(&self) -> u64 {
        self.games.load(Ordering::Relaxed)
    } 

    fn wins(&self) -> u64 {
        self.white_wins.load(Ordering::Relaxed)
    } 

    fn draws(&self) -> u64 {
        self.draws.load(Ordering::Relaxed)
    }

    fn loses(&self) -> u64 {
        self.games() - self.wins() - self.draws()
    }

    fn avg_iterations(&self) -> f64 {
        self.iterations.load(Ordering::Relaxed) as f64 / self.games().max(1) as f64
    }

    pub fn add_game(&self, positions: u64, result: i32, avg_iters: u64) {
        self.games.fetch_add(1, Ordering::Relaxed);
        self.positions.fetch_add(positions, Ordering::Relaxed);
        self.iterations.fetch_add(avg_iters, Ordering::Relaxed);

        match result {
            1 => self.white_wins.fetch_add(1, Ordering::Relaxed),
            0 => self.draws.fetch_add(1, Ordering::Relaxed),
            _ => 0
        };
    }
}