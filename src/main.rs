use std::{env, io::stdin};
use processors::{MiscCommandsProcessor, ParamsProcessor, UciProcessor};

mod processors;
mod utils;
mod search;

fn main() {

    //Process arguments passed when starting the engine
    if ParamsProcessor::execute(env::args().collect()) {
        return;
    }

    println!("Jackal v{} by Tomasz Jaworski\n", env!("CARGO_PKG_VERSION"));

    loop {

        //Reading the input to obtain command
        let mut input_command = String::new();
        if stdin().read_line(&mut input_command).is_err() {
            println!("Error reading input, please try again.");
            continue;
        }

        //Parsing command string and skipping if command is empty
        let input_command = input_command.trim();
        let parts: Vec<&str> = input_command.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        //Split command string into command and it's arguments, and progress misc commands
        let command = parts[0];
        let args = &parts[1..].iter().map(|arg_str| arg_str.to_string()).collect::<Vec<String>>();
        if MiscCommandsProcessor::execute(command, args) {
            break;
        }

        //Process UCI protocol commands
        UciProcessor::execute(command, args);
    }
}