use std::cmp::Ordering;

use spear::{Move, Perft, Side, FEN};

use crate::{
    search::{NodeIndex, PolicyNetwork, SearchEngine},
    utils::{clear_terminal_screen, heat_color},
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

    //Prints all legal moves together with thier policy
    fn moves(search_engine: &SearchEngine) {
        println!("All legal moves");
        let mut moves: Vec<(Move, f32)> = Vec::new();
        let board = *search_engine.current_position().board();
        let mut inputs: Vec<usize> = Vec::with_capacity(32);

        let vertical_flip = if board.side_to_move() == Side::WHITE {
            0
        } else {
            56
        };

        let mut max = f32::NEG_INFINITY;
        if search_engine.current_position().board().side_to_move() == Side::WHITE {
            PolicyNetwork::map_policy_inputs::<_, true, false>(&board, |idx| inputs.push(idx) );
            board.map_moves::<_, true, false>(|mv| {
                let policy = PolicyNetwork.forward(&inputs, mv, vertical_flip);
                max = max.max(policy);
                moves.push((mv, policy))
            })
        } else {
            PolicyNetwork::map_policy_inputs::<_, false, true>(&board, |idx| inputs.push(idx) );
            board.map_moves::<_, false, true>(|mv| {
                let policy = PolicyNetwork.forward(&inputs, mv, vertical_flip);
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
            *policy = if is_single_action { 1.0 } else { *policy / total };
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
}
