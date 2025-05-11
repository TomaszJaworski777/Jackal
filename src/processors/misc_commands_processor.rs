use std::cmp::Ordering;

use crate::bench::Bench;
use crate::spear::{ChessBoard, ChessPosition, Move, Perft, Piece, Side, FEN};

use crate::{
    search::{NodeIndex, PolicyNetwork, Score, SearchEngine, ValueNetwork},
    utils::{clear_terminal_screen, heat_color},
    EngineOptions,
};

pub struct MiscCommandsProcessor;
impl MiscCommandsProcessor {
    //Handles commands that are not UCI related and returns true, if command was found, so it can skip uci commands
    pub fn execute(command: &str, args: &[String], search_engine: &SearchEngine) -> bool {
        match command {
            "exit" | "quit" | "q" => std::process::exit(0),
            "clean" | "clear" | "cls" | "cln" => clear_terminal_screen(),
            "perft" => {
                Self::perft::<false>(args, &search_engine.current_position().board().get_fen())
            }
            "bulk" => {
                Self::perft::<true>(args, &search_engine.current_position().board().get_fen())
            }
            "draw" | "d" => search_engine.current_position().board().draw_board(),
            "moves" => Self::moves(search_engine),
            "tree" => Self::draw_tree(args, search_engine),
            "eval" | "e" => Self::eval(search_engine),
            "bench" => Self::bench(args, search_engine),
            _ => return false,
        }

        true
    }

    //Performs move generator performance test
    fn perft<const BULK: bool>(args: &[String], current_fen: &FEN) {
        //Obtain test depth from command arguments
        let depth = if args.is_empty() {
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

    //Performs performance test of search engine
    fn bench(args: &[String], search_engine: &SearchEngine) {
        //Obtain test depth from command arguments
        let depth = if args.is_empty() {
            5
        } else {
            let parse = args[0].parse::<u32>();
            if parse.is_err() {
                return;
            }
            parse.unwrap()
        };

        Bench::run(depth, search_engine);
    }

    //Prints all legal moves together with thier policy
    fn moves(search_engine: &SearchEngine) {
        println!("All legal moves");
        let mut moves: Vec<(Move, f32)> = Vec::new();
        let board = *search_engine.current_position().board();

        let mut max = f32::NEG_INFINITY;
        if search_engine.current_position().board().side_to_move() == Side::WHITE {
            let base = PolicyNetwork.create_base::<true, false>(&board);
            board.map_moves::<_, true, false>(|mv| {
                let policy = PolicyNetwork.forward::<true, false>(
                    &board,
                    &base,
                    mv,
                ) + mva_lvv(mv, &board, search_engine.engine_options());
                max = max.max(policy);
                moves.push((mv, policy))
            })
        } else {
            let base = PolicyNetwork.create_base::<false, true>(&board);
            board.map_moves::<_, false, true>(|mv| {
                let policy = PolicyNetwork.forward::<false, true>(
                    &board,
                    &base,
                    mv,
                ) + mva_lvv(mv, &board, search_engine.engine_options());
                max = max.max(policy);
                moves.push((mv, policy))
            })
        }

        let mut total = 0.0;
        for (_, policy) in &mut moves {
            *policy = (*policy - max).exp();
            total += *policy;
        }

        let is_single_action = moves.len() == 1;
        for (_, policy) in &mut moves {
            *policy = if is_single_action {
                1.0
            } else {
                *policy / total
            };
        }

        let max_policy = moves
            .iter()
            .max_by(|&a, &b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap()
            .1;
        let min_policy = moves
            .iter()
            .min_by(|&a, &b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap()
            .1;

        moves.sort_by(|&(_, a), &(_, b)| b.partial_cmp(&a).unwrap_or(Ordering::Equal));

        for (index, &(mv, policy)) in moves.iter().enumerate() {
            let arrow = if index == moves.len() - 1 {
                "└─>"
            } else {
                "├─>"
            };
            println!(
                "{} {:<4} {} - {}",
                arrow,
                format!("{}.", index + 1),
                mv,
                heat_color(
                    format!("{:.2}%", policy * 100.0).as_str(),
                    policy,
                    min_policy,
                    max_policy
                )
            )
        }
    }

    fn draw_tree(args: &[String], search_engine: &SearchEngine) {
        if args.len() > 3 {
            return;
        }

        let depth = if !args.is_empty() {
            args[0].parse::<u32>().unwrap_or(1).max(1)
        } else {
            1
        };

        let node_index = if args.len() == 3 {
            let segment = args[1]
                .replace(['(', ','], "")
                .trim()
                .parse::<u32>()
                .expect("Incorrect segment");
            let index = args[2]
                .replace(')', "")
                .trim()
                .parse::<u32>()
                .expect("Incorrect index");
            NodeIndex::from_parts(index, segment)
        } else {
            search_engine.tree().root_index()
        };

        if search_engine.current_position().board().side_to_move() == Side::WHITE {
            search_engine.tree().draw_tree::<true, false>(
                search_engine.current_position().board(),
                node_index,
                depth,
            )
        } else {
            search_engine.tree().draw_tree::<false, true>(
                search_engine.current_position().board(),
                node_index,
                depth,
            )
        }
    }

    fn eval(search_engine: &SearchEngine) {
        let position: &ChessPosition = &search_engine.current_position();
        let (w, d, _) = if position.board().side_to_move() == Side::WHITE {
            ValueNetwork.forward::<true, false>(position.board())
        } else {
            ValueNetwork.forward::<false, true>(position.board())
        };
        let score = Score::new(w, d);

        let draw_score = search_engine.engine_options().draw_score();

        position.board().draw_board();
        println!("For me");
        println!("Score: {} ({:.2})", score.single(draw_score), score.as_cp_f32(draw_score));
        println!("WDL: [{:.2}%, {:.2}%, {:.2}%]\n", score.win_chance() * 100.0, score.draw_chance() * 100.0, score.lose_chance() * 100.0);

        let draw_score = search_engine.engine_options().draw_score_opp();

        println!("For them");
        println!("Score: {} ({:.2})", score.single(draw_score), score.as_cp_f32(draw_score));
        println!("WDL: [{:.2}%, {:.2}%, {:.2}%]\n", score.win_chance() * 100.0, score.draw_chance() * 100.0, score.lose_chance() * 100.0);
    }
}

const MVA_LVV_PIECE_VALUES: [f32; 5] = [1.0, 3.0, 3.0, 5.0, 9.0];
fn mva_lvv(mv: Move, board: &ChessBoard, options: &EngineOptions) -> f32 {
    let attacker = board.get_piece_on_square(mv.get_from_square());
    let victim = board.get_piece_on_square(mv.get_to_square());

    if !mv.is_capture() || victim == Piece::NONE || attacker == Piece::KING {
        return 0.0;
    }

    (MVA_LVV_PIECE_VALUES[attacker.get_raw() as usize]
        - MVA_LVV_PIECE_VALUES[victim.get_raw() as usize])
        * options.policy_sac_bonus()
}
