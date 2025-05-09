use std::{
    io::stdin,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::spear::{ChessBoard, ChessPosition, FEN};

use crate::options::EngineOptions;

use super::{
    print::{NoPrint, PrettyPrint, UciPrint},
    search_limits::SearchLimits,
    tree::Tree,
    utils::ContemptParams,
    Mcts, SearchStats,
};

pub struct SearchEngine<'a> {
    position: ChessPosition,
    previous_board: ChessBoard,
    interruption_token: &'a AtomicBool,
    tree: &'a mut Tree,
    options: &'a mut EngineOptions,
    command_queue: &'a mut Vec<String>,
    uci_initialized: bool,
    game_ply: u32,
}

impl<'a> SearchEngine<'a> {
    pub fn new(
        position: ChessPosition,
        interruption_token: &'a AtomicBool,
        tree: &'a mut Tree,
        options: &'a mut EngineOptions,
        command_queue: &'a mut Vec<String>,
    ) -> Self {
        Self {
            position,
            previous_board: *position.board(),
            interruption_token,
            tree,
            options,
            command_queue,
            uci_initialized: false,
            game_ply: 0,
        }
    }

    pub fn init_uci(&mut self) {
        self.uci_initialized = true;
    }

    pub fn command_queue(&mut self) -> &mut Vec<String> {
        self.command_queue
    }

    pub fn engine_options(&self) -> &EngineOptions {
        self.options
    }

    pub fn engine_options_mut(&mut self) -> &mut EngineOptions {
        self.options
    }

    pub fn replace_position(&mut self, position: ChessPosition) {
        self.position = position
    }

    pub fn current_position(&self) -> ChessPosition {
        self.position
    }

    pub fn game_ply(&self) -> u32 {
        self.game_ply
    }

    pub fn tree(&self) -> &Tree {
        self.tree
    }

    pub fn tree_mut(&mut self) -> &mut Tree {
        self.tree
    }

    pub fn reset(&mut self) {
        self.position = ChessPosition::from_fen(&FEN::start_position());
        self.tree.clear();
        self.game_ply = 0;
    }

    pub fn search(&mut self, search_limits: &SearchLimits, print_reports: bool) {
        //Init values for the search
        self.interruption_token.store(false, Ordering::Relaxed);
        let search_stats = SearchStats::new();
        self.tree
            .reuse_tree(&self.previous_board, self.current_position().board());
        self.previous_board = *self.current_position().board();

        self.game_ply += 2;

        let contempt_parms = ContemptParams::calculate_params(self.options);

        if self.options.analyse_mode() {
            self.tree.clear();
        }

        //Start the search thread
        std::thread::scope(|s| {
            s.spawn(|| {
                let mcts = Mcts::new(
                    self.position,
                    self.tree,
                    self.interruption_token,
                    self.options,
                    &search_stats,
                    search_limits,
                    &contempt_parms,
                );
                let (_, _) = if print_reports {
                    if self.uci_initialized {
                        mcts.search::<UciPrint>()
                    } else {
                        mcts.search::<PrettyPrint>()
                    }
                } else {
                    mcts.search::<NoPrint>()
                };
            });

            //Create portable loop to handle command queue during search
            Self::portable_command_handler(self.command_queue, self.interruption_token)
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
