use std::{
    io::stdin,
    sync::atomic::{AtomicBool, Ordering},
};

use spear::{ChessPosition, FEN};

use crate::options::EngineOptions;

use super::{print::{NoPrint, UciPrint}, search_limits::SearchLimits, tree::SearchTree, Mcts, SearchStats};

pub struct SearchEngine<'a> {
    position: ChessPosition,
    interruption_token: &'a AtomicBool,
    tree: &'a mut SearchTree,
    options: &'a mut EngineOptions,
    command_queue: &'a mut Vec<String>,
    uci_initialized: bool,
}

impl<'a> SearchEngine<'a> {
    pub fn new(
        position: ChessPosition,
        interruption_token: &'a AtomicBool,
        tree: &'a mut SearchTree,
        options: &'a mut EngineOptions,
        command_queue: &'a mut Vec<String>,
    ) -> Self {
        Self {
            position,
            interruption_token,
            tree,
            options,
            command_queue,
            uci_initialized: false,
        }
    }

    pub fn init_uci(&mut self) {
        self.uci_initialized = true;
    }

    pub fn command_queue(&mut self) -> &mut Vec<String> {
        &mut self.command_queue
    }

    pub fn engine_options(&self) -> &EngineOptions {
        &self.options
    }

    pub fn engine_options_mut(&mut self) -> &mut EngineOptions {
        &mut self.options
    }

    pub fn replace_position(&mut self, position: ChessPosition) {
        self.position = position
    }

    pub fn current_position(&self) -> ChessPosition {
        self.position
    }

    pub fn reset(&mut self) {
        self.position = ChessPosition::from_fen(&FEN::start_position());
        self.tree.clear();
    }

    pub fn stop(&self) {
        self.interruption_token.store(true, Ordering::Relaxed)
    }

    pub fn search(&mut self, search_limits: &SearchLimits, print_reports: bool) {
        //Init values for the search
        self.interruption_token.store(false, Ordering::Relaxed);
        let search_stats = SearchStats::new();
        self.tree.clear();

        //Start the search thread
        std::thread::scope(|s| {
            s.spawn(|| {
                let mut mcts = Mcts::new(
                    self.position.clone(),
                    self.tree,
                    self.interruption_token,
                    self.options,
                    &search_stats,
                    search_limits,
                );
                let (best_move, best_score) = if print_reports {
                    if self.uci_initialized {
                        mcts.search::<UciPrint>()
                    } else {
                        mcts.search::<UciPrint>()
                    }
                } else {
                    mcts.search::<NoPrint>()
                };
            });

            //Create portable loop to handle command queue during search
            Self::portable_command_handler(&mut self.command_queue, &self.interruption_token)
        });
    }

    fn portable_command_handler(command_queue: &mut Vec<String>, interruption_token: &AtomicBool) {
        loop {
            let mut input_command = String::new();
            if stdin().read_line(&mut input_command).is_err() {
                println!("Error reading input, please try again.");
                continue;
            }

            let input_command = input_command.trim();

            match input_command {
                "isready" => println!("readyok"),
                "quit" | "q" | "exit" => std::process::exit(0),
                "stop" => interruption_token.store(true, Ordering::Relaxed),
                _ => command_queue.push(input_command.to_string()),
            }

            if interruption_token.load(Ordering::Relaxed) {
                break;
            }
        }
    }
}
