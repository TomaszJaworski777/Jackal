use std::sync::atomic::AtomicBool;

use spear::{ChessPosition, Move, Side, FEN};

use crate::{
    options::EngineOptions,
    search::{SearchEngine, SearchLimits},
};

pub struct UciProcessor;
impl UciProcessor {
    //Handles commands from UCI protocol
    pub fn execute(command: &str, args: &[String], search_engine: &mut SearchEngine) {
        match command {
            "uci" => Self::uci_message(search_engine),
            "isready" => println!("readyok"),
            "setoption" => Self::set_option(args, search_engine.engine_options_mut()),
            "position" => Self::position(args, search_engine),
            "ucinewgame" => search_engine.reset(),
            "go" => Self::go(args, search_engine),
            _ => return,
        }
    }

    fn uci_message(search_engine: &mut SearchEngine) {
        println!("id name Jackal v{}", env!("CARGO_PKG_VERSION"));
        println!("id author Tomasz Jaworski");
        search_engine.engine_options().print();
        println!("uciok");

        search_engine.init_uci()
    }

    fn set_option(args: &[String], options: &mut EngineOptions) {
        //Checks if command was initially correct
        if args.len() != 4 || args[0] != "name" || args[2] != "value" {
            return;
        }

        //Tries to execute the set option command
        options.set(args[1].as_str(), args[3].as_str())
    }

    fn position(args: &[String], search_engine: &mut SearchEngine) {
        let mut move_flag = false;
        let mut fen_flag = false;
        let mut fen = String::new();
        let mut moves = Vec::new();

        //Parse the command arguments into fen and move list
        for arg in args {
            match arg.as_str() {
                "startpos" => fen = FEN::start_position().to_string(),
                "fen" => {
                    fen_flag = true;
                    move_flag = false;
                }
                "moves" => {
                    fen_flag = false;
                    move_flag = true;
                }
                _ => {
                    if fen_flag {
                        fen.push_str(&format!("{arg} "))
                    }

                    if move_flag {
                        moves.push(arg)
                    }
                }
            }
        }

        //Prepare method that will map move strings from move list into actual confirmed legal moves
        fn map_move<const STM_WHITE: bool, const NSTM_WHITE: bool>(
            mv: &String,
            chess_position: &mut ChessPosition,
        ) {
            let mut _mv = Move::NULL;
            chess_position
                .board()
                .map_moves::<_, STM_WHITE, NSTM_WHITE>(|legal_mv| {
                    if *mv == legal_mv.to_string() {
                        _mv = legal_mv;
                        return;
                    }
                });
            chess_position.make_move::<STM_WHITE, NSTM_WHITE>(_mv);
        }

        //Prepare chess position based on parsed fen and map the moves into it
        let mut chess_position = ChessPosition::from_fen(&FEN::from_string(fen));
        for mv in moves {
            if chess_position.board().side_to_move() == Side::WHITE {
                map_move::<true, false>(mv, &mut chess_position)
            } else {
                map_move::<false, true>(mv, &mut chess_position)
            }
        }

        //Set new position
        search_engine.replace_position(chess_position)
    }

    fn go(args: &[String], search_engine: &mut SearchEngine) {
        let mut search_limits = SearchLimits::new();

        let parse_u128 = |str: &str| -> u128 { str.parse::<u128>().unwrap_or(0).max(0) };

        let parse_u32 = |str: &str| -> u32 { str.parse::<u32>().unwrap_or(0).max(0) };

        //Convert args into search parameters
        if args.len() == 0 {
            search_limits.go_infinite()
        } else {
            let mut search_param = "";
            let param_list = [
                "wtime",
                "btime",
                "winc",
                "binc",
                "movestogo",
                "depth",
                "nodes",
                "infinite",
                "movetime",
            ];
            let side_to_move = search_engine.current_position().board().side_to_move();
            for arg in args {
                if param_list.contains(&arg.as_str()) {
                    search_param = arg;
                    if search_param == "infinite" {
                        search_limits.go_infinite()
                    }
                    continue;
                }

                match search_param {
                    "wtime" => {
                        if side_to_move == Side::WHITE {
                            search_limits.add_time_remaining(parse_u128(search_param))
                        }
                    }
                    "btime" => {
                        if side_to_move == Side::BLACK {
                            search_limits.add_time_remaining(parse_u128(search_param))
                        }
                    }
                    "winc" => {
                        if side_to_move == Side::WHITE {
                            search_limits.add_increment(parse_u128(search_param))
                        }
                    }
                    "binc" => {
                        if side_to_move == Side::BLACK {
                            search_limits.add_increment(parse_u128(search_param))
                        }
                    }
                    "movestogo" => search_limits.add_moves_to_go(parse_u32(search_param)),
                    "movetime" => search_limits.add_move_time(parse_u128(search_param)),
                    "depth" => search_limits.add_depth(parse_u32(search_param)),
                    "nodes" => search_limits.add_iters(parse_u32(search_param)),
                    _ => continue,
                }
            }
        }

        search_engine.search(&search_limits, true);
    }
}
