use spear::{Perft, FEN};

use crate::{search::SearchEngine, utils::clear_terminal_screen};

pub struct MiscCommandsProcessor;
impl MiscCommandsProcessor {
        
    //Handles commands that are not UCI related and returns true, if command was found, so it can skip uci commands
    pub fn execute( command: &str, args: &[String], search_engine: &SearchEngine ) -> bool {
        match command {
            "exit" | "quit" | "q" => std::process::exit(0),
            "clean" | "clear" | "cls" | "cln" => clear_terminal_screen(),
            "perft" => Self::perft::<false>(args, &search_engine.current_position().board().get_fen()),
            "bulk" => Self::perft::<true>(args, &search_engine.current_position().board().get_fen()),
            "draw" | "d" => search_engine.current_position().board().draw_board(),
            _ => return false
        }

        true
    }

    //Performs move generator performance test
    fn perft<const BULK: bool>(args: &[String], current_fen: &FEN) {

        //Obtain test depth from command arguments
        let depth = if args.len() == 0 {
            5
        } else {
            let parse = args[0].parse::<u8>();
            if parse.is_err() {
                return;
            }
            parse.unwrap()
        };

        Perft::perft::<BULK, true, true>(current_fen, depth);
    }
}