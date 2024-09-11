use options::EngineOptions;
use processors::{MiscCommandsProcessor, ParamsProcessor, UciProcessor};
use search::{SearchEngine, SearchTree};
use spear::{ChessPosition, FEN};
use std::{env, io::stdin, sync::atomic::AtomicBool};

mod options;
mod processors;
mod search;
mod utils;

fn main() {
    //Process arguments passed when starting the engine
    if ParamsProcessor::execute(env::args().collect()) {
        return;
    }

    //Init search engine
    let start_position = ChessPosition::from_fen(&FEN::start_position());
    let interruption_token = AtomicBool::new(false);
    let mut tree = SearchTree::new();
    let mut options = EngineOptions::new();
    let mut command_queue: Vec<String> = Vec::new();
    let mut search_engine = SearchEngine::new(
        start_position,
        &interruption_token,
        &mut tree,
        &mut options,
        &mut command_queue,
    );

    println!("Jackal v{} by Tomasz Jaworski\n", env!("CARGO_PKG_VERSION"));

    loop {
        //Reading the input to obtain command
        let queue = search_engine.command_queue();
        let input_command = if queue.len() > 0 {
            queue.remove(0)
        } else {
            let mut input_command = String::new();
            if stdin().read_line(&mut input_command).is_err() {
                println!("Error reading input, please try again.");
                continue;
            }

            input_command
        };

        //Parsing command string and skipping if command is empty
        let input_command = input_command.trim();
        let parts: Vec<&str> = input_command.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        //Split command string into command and it's arguments, and progress misc commands
        let command = parts[0];
        let args = &parts[1..]
            .iter()
            .map(|arg_str| arg_str.to_string())
            .collect::<Vec<String>>();
        if MiscCommandsProcessor::execute(command, args, &search_engine) {
            continue;
        }

        //Process UCI protocol commands
        UciProcessor::execute(command, args, &mut search_engine);
    }
}
