use std::io::Write;

use chess::{ChessBoard, ChessPosition, Piece, Side, Square, DEFAULT_PERFT_DEPTH, FEN};
use engine::{NoReport, NodeIndex, PolicyNetwork, SearchEngine, SearchLimits, ValueNetwork, WDLScore};
use utils::{clear_terminal_screen, create_loading_bar, heat_color, time_to_string, number_to_string, AlignString, Colors, CustomColor, PieceColors, Theme, DRAW_COLOR, LOSE_COLOR, WIN_COLOR};

pub struct MiscProcessor;
impl MiscProcessor {
    pub fn execute(
        command: &str,
        args: &[String],
        search_engine: &mut SearchEngine,
        shutdown_token: &mut bool,
    ) -> bool {
        match command {
            "exit" | "quit" | "q" => *shutdown_token = true,
            "clear" | "clean" | "cls" => clear_terminal_screen(),
            "draw" | "d" => search_engine.root_position().board().draw_board(),
            "tree" => {
                let depth = if args.len() >= 1 {
                    args[0].parse::<u8>().ok()
                } else {
                    None
                };
                let node_idx = if args.len() >= 3 {
                    let half = args[1].replace("(", "").replace(",", "").parse::<u64>().ok();
                    let idx = args[2].replace(")", "").parse::<u64>().ok();

                    if half.is_none() || idx.is_none() {
                        None
                    } else {
                        Some(NodeIndex::new(half.unwrap(), idx.unwrap()))
                    }
                } else {
                    None
                };
                
                search_engine.tree().draw_tree::<true>(depth, node_idx,&search_engine);
            },
            "rawtree" => {
                let depth = if args.len() >= 1 {
                    args[0].parse::<u8>().ok()
                } else {
                    None
                };
                let node_idx = if args.len() >= 3 {
                    let half = args[1].replace("(", "").replace(",", "").parse::<u64>().ok();
                    let idx = args[2].replace(")", "").parse::<u64>().ok();

                    if half.is_none() || idx.is_none() {
                        None
                    } else {
                        Some(NodeIndex::new(half.unwrap(), idx.unwrap()))
                    }
                } else {
                    None
                };

                search_engine.tree().draw_tree::<false>(depth, node_idx, &search_engine);
            },
            "perft" => {
                let depth = if args.len() >= 1 {
                    args[0].parse::<u8>().ok()
                } else {
                    None
                };
                perft::<false, false>(search_engine, depth);
            },
            "bulk" => {
                let depth = if args.len() >= 1 {
                    args[0].parse::<u8>().ok()
                } else {
                    None
                };
                perft::<true, false>(search_engine, depth);
            },
            "bench" => {
                let depth = if args.len() >= 1 {
                    args[0].parse::<u64>().ok()
                } else {
                    None
                };
                let (result, duration) = search_engine.bench(depth);
                let nps = result as f64 / duration.as_secs_f64();
                println!("Bench: {result} nodes {:.0} nps", nps);
            },
            "eval-bench" => eval_bench(),
            "policy" => draw_policy(search_engine),
            "eval" => eval(search_engine),
            "analyse" => {
                let iters = if args.len() >= 1 {
                    args[0].parse::<u64>().ok()
                } else {
                    None
                };

                analyse(search_engine, iters);
            },
            _ => return false,
        }

        true
    }
}

fn perft<const BULK: bool, const CHESS_960: bool>(search_engine: &SearchEngine, depth: Option<u8>) {
    println!("");

    search_engine.root_position().board().draw_board();

    println!("-----------------------------------------------------------");
    println!("  Running PERFT");
    println!("  Depth: {}", depth.unwrap_or(DEFAULT_PERFT_DEPTH));
    println!("  Bulk: {BULK}");
    println!("-----------------------------------------------------------\n");

    let (result, duration) =
        chess::perft::<BULK, true, CHESS_960>(search_engine.root_position().board(), depth);
    let miliseconds = duration.as_millis().max(1);

    println!("\n-----------------------------------------------------------");
    println!(
        "  Perft ended! {result} nodes, {}, {}n/s",
        time_to_string(miliseconds),
        number_to_string(((result * 1000) as f64 / miliseconds as f64) as u128)
    );
    println!("-----------------------------------------------------------\n");
}

fn eval_bench() {
    const FENS: [&str; 5] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    ];

    for fen in FENS {
        let board = ChessBoard::from(&FEN::from(fen));
        let wdl_score = ValueNetwork.forward(&board);
        println!("{}",
            format!("{fen}: {}", 
                format!("[{}, {}, {}]",
                    format!("{:.2}%", wdl_score.win_chance() * 100.0).custom_color(WIN_COLOR),
                    format!("{:.2}%", wdl_score.draw_chance() * 100.0).custom_color(DRAW_COLOR),
                    format!("{:.2}%", wdl_score.lose_chance() * 100.0).custom_color(LOSE_COLOR),
                ).secondary(10.0/18.0)
            ).primary(10.0/18.0)
        )
    }
}

fn draw_policy(search_engine: &SearchEngine) {
    let board = search_engine.root_position().board();

    board.draw_board();

    let policy_base = PolicyNetwork.create_base(board);
    let mut max = f32::NEG_INFINITY;
    let mut total = 0f32;

    let mut min_policy = f32::INFINITY;
    let mut max_policy = f32::NEG_INFINITY;
    let mut moves = Vec::new();

    board.map_legal_moves(|mv| {
        let see = board.see(mv, -108);
        let p = PolicyNetwork.forward(board, &policy_base, mv, see, search_engine.options().chess960());
        max = max.max(p);
        moves.push((mv, p));
    });

    for (_, p) in moves.iter_mut() {
        *p = (*p - max).exp();
        total += *p;
    }

    for (_, p) in moves.iter_mut() {
        *p = *p / total;
        min_policy = min_policy.min(*p);
        max_policy = max_policy.max(*p);
    }

    if moves.len() == 1 {
        moves[0].1 = 1.0
    }

    moves.sort_by(|&a, &b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (idx, &(mv, p)) in moves.iter().enumerate() {
        println!(" {} {}", 
            format!("{}:", mv.to_string(search_engine.options().chess960())).align_to_left(6).primary((idx as f32 + 10.0)/(moves.len() as f32 + 18.0)), 
            heat_color(&format!("{:.2}%", p * 100.0), p, min_policy, max_policy)
        )
    }
}

fn eval(search_engine: &SearchEngine) {
    let board = search_engine.root_position().board();
    board.draw_board();

    let raw_score = ValueNetwork.forward(board);
    let raw_score_cp = raw_score.cp();

    println!("{}\n", format!("Raw: {}", 
        format!("[{}, {}, {}] ({}{})",
            format!("{:.2}%", raw_score.win_chance() * 100.0).custom_color(WIN_COLOR),
            format!("{:.2}%", raw_score.draw_chance() * 100.0).custom_color(DRAW_COLOR),
            format!("{:.2}%", raw_score.lose_chance() * 100.0).custom_color(LOSE_COLOR),
            heat_color(if raw_score_cp > 0 { "+" } else { "-" }, raw_score.single() as f32, 0.0, 1.0),
            heat_color(format!("{:.2}", raw_score_cp.abs() as f32 / 100.0).as_str(), raw_score.single() as f32, 0.0, 1.0),
        ).secondary(10.0/32.0)
    ).primary(10.0/32.0));

    let mut half_moves = raw_score;
    half_moves.apply_50mr_and_draw_scaling(board.half_moves(), 0.0, search_engine.options());
    let half_moves_cp = half_moves.cp();

    println!("{}", format!("50mr scaling:     {}", 
        format!("[{}, {}, {}] ({}{})",
            format!("{:.2}%", half_moves.win_chance() * 100.0).custom_color(WIN_COLOR),
            format!("{:.2}%", half_moves.draw_chance() * 100.0).custom_color(DRAW_COLOR),
            format!("{:.2}%", half_moves.lose_chance() * 100.0).custom_color(LOSE_COLOR),
            heat_color(if half_moves_cp > 0 { "+" } else { "-" }, half_moves.single() as f32, 0.0, 1.0),
            heat_color(format!("{:.2}", half_moves_cp.abs() as f32 / 100.0).as_str(), half_moves.single() as f32, 0.0, 1.0),
        ).secondary(14.0/32.0)
    ).primary(14.0/32.0));

    let mut material_scaling = raw_score;
    material_scaling.apply_material_scaling(board, search_engine.options());
    let material_scaling_cp = material_scaling.cp();

    println!("{}", format!("Material scaling: {}", 
        format!("[{}, {}, {}] ({}{})",
            format!("{:.2}%", material_scaling.win_chance() * 100.0).custom_color(WIN_COLOR),
            format!("{:.2}%", material_scaling.draw_chance() * 100.0).custom_color(DRAW_COLOR),
            format!("{:.2}%", material_scaling.lose_chance() * 100.0).custom_color(LOSE_COLOR),
            heat_color(if material_scaling_cp > 0 { "+" } else { "-" }, material_scaling.single() as f32, 0.0, 1.0),
            heat_color(format!("{:.2}", material_scaling_cp.abs() as f32 / 100.0).as_str(), material_scaling.single() as f32, 0.0, 1.0),
        ).secondary(16.0/32.0)
    ).primary(16.0/32.0));

    let mut pre_contempt = raw_score;
    pre_contempt.apply_50mr_and_draw_scaling(board.half_moves(), 0.0, search_engine.options());
    pre_contempt.apply_material_scaling(board, search_engine.options());
    let pre_contempt_cp = pre_contempt.cp();

    println!("{}", format!("Before Contempt:  {}", 
        format!("[{}, {}, {}] ({}{})",
            format!("{:.2}%", pre_contempt.win_chance() * 100.0).custom_color(WIN_COLOR),
            format!("{:.2}%", pre_contempt.draw_chance() * 100.0).custom_color(DRAW_COLOR),
            format!("{:.2}%", pre_contempt.lose_chance() * 100.0).custom_color(LOSE_COLOR),
            heat_color(if pre_contempt_cp > 0 { "+" } else { "-" }, pre_contempt.single() as f32, 0.0, 1.0),
            heat_color(format!("{:.2}", pre_contempt_cp.abs() as f32 / 100.0).as_str(), pre_contempt.single() as f32, 0.0, 1.0),
        ).secondary(18.0/32.0)
    ).primary(18.0/32.0));

    let mut v = pre_contempt.win_chance() - pre_contempt.lose_chance();
    let mut d = pre_contempt.draw_chance();

    let total_score = WDLScore::new((1.0 + v - d) / 2.0, d);
    let total_score_cp = total_score.cp();

    println!("{}", format!("After Contempt:   {}", 
        format!("[{}, {}, {}] ({}{})",
            format!("{:.2}%", total_score.win_chance() * 100.0).custom_color(WIN_COLOR),
            format!("{:.2}%", total_score.draw_chance() * 100.0).custom_color(DRAW_COLOR),
            format!("{:.2}%", total_score.lose_chance() * 100.0).custom_color(LOSE_COLOR),
            heat_color(if total_score_cp > 0 { "+" } else { "-" }, total_score.single() as f32, 0.0, 1.0),
            heat_color(format!("{:.2}", total_score_cp.abs() as f32 / 100.0).as_str(), total_score.single() as f32, 0.0, 1.0),
        ).secondary(20.0/32.0)
    ).primary(20.0/32.0));
}

fn analyse(search_engine: &mut SearchEngine, iters: Option<u64>) {
    let position = *search_engine.root_position();
    let board = *position.board();
    let iters = iters.unwrap_or(50000);

    let mut search_limits = SearchLimits::default();
    search_limits.set_iters(Some(iters));

    println!("\n{} {}", " FEN:   ".primary(0.0/32.0), FEN::from(&board).to_string().secondary(0.0/32.0));
    println!("{} {}\n", " Nodes: ".primary(1.0/32.0), iters.to_string().secondary(1.0/32.0));

    let piece_count = board.occupancy().pop_count() - 2 + 1;
    let mut progress = 0.0;

    print!("{} {}\r", " Progress:".primary(4.0/32.0), create_loading_bar(50, progress, (225,225,225), (225,225,225)).secondary(4.0/32.0));
    let _ = std::io::stdout().flush();

    search_engine.search::<NoReport>(&search_limits);
    progress += 1.0;

    print!("{} {}\r", " Progress:".primary(5.0/32.0), create_loading_bar(50, progress / piece_count as f32, (225,225,225), (225,225,225)).secondary(5.0/32.0));
    let _ = std::io::stdout().flush();

    let draw_score = search_engine.options().draw_score() as f64 / 100.0;
    let wdl_score = search_engine.tree().get_best_pv(0, draw_score).score();
    let current_eval = wdl_score.cp();

    let info = [const { String::new() }; 33];

    let mut evals = [0; 64];
    board.occupancy().map(|square| {
        let piece = board.piece_on_square(square);
        let side = board.color_on_square(square);

        if piece == Piece::NONE || piece == Piece::KING {
            return;
        }

        progress += 1.0;

        let mut board_cpy = board;
        board_cpy.remove_piece_on_square(square, piece, side);

        if board_cpy.is_square_attacked(board.king_square(board.side().flipped()), board.side().flipped()) {
            print!("{} {}\r", " Progress:".primary(5.0/32.0), create_loading_bar(50, progress / piece_count as f32, (225,225,225), (225,225,225)).secondary(5.0/32.0));
            let _ = std::io::stdout().flush();
            return;
        }

        search_engine.set_position(&ChessPosition::from(board_cpy), 0);
        search_engine.search::<NoReport>(&search_limits);

        evals[usize::from(square)] = search_engine.tree().get_best_pv(0, draw_score).score().cp();

        print!("{} {}\r", " Progress:".primary(5.0/32.0), create_loading_bar(50, progress / piece_count as f32, (225,225,225), (225,225,225)).secondary(5.0/32.0));
        let _ = std::io::stdout().flush();
    });

    print!("{}\r", " ".repeat(68));
    let _ = std::io::stdout().flush();

    draw_eval_board(&board, &info, current_eval, evals);

    search_engine.set_position(&position, 0);
}

fn draw_eval_board(board: &ChessBoard, info: &[String; 33], current_eval: i32, board_evals: [i32; 64]) {
    for rank in 0..9 {
        for file in 0..8 {
            print!("{}", "+-------".primary(rank as f32 * 4.0 / 32.0));
            if file == 7 {
                println!("{}   {}", 
                    "+".primary((rank * 4) as f32 / 32.0),
                    info[(rank * 4) as usize]
                )
            }
        }

        if rank == 8 {
            break;
        }

        let square_data = |row_idx: u8, rank: u8, file: u8| -> String {
            let square = Square::from_coords(rank, file);

            let piece = board.piece_on_square(square);
            let side = board.color_on_square(square);

            match row_idx {
                0 => {
                    let file_char = if rank == if board.side() == Side::WHITE { 0 } else { 7 } {
                        (b'A' + square.file()) as char
                    } else {
                        ' '
                    };

                    let rank_str = if file == if board.side() == Side::WHITE { 0 } else { 7 } {
                        (square.get_rank() + 1).to_string()
                    } else {
                        String::from(" ")
                    };

                    format!("{}{}{}", rank_str, " ".repeat(5), file_char).align_to_center(7).gray()
                },
                1 => {
                    if board.en_passant_square() == square {
                        return "x".align_to_center(7);
                    }

                    let piece_char = char::from(piece).to_string();
                    
                    if side == Side::WHITE {
                        piece_char.to_ascii_uppercase().align_to_center(7).white_pieces()
                    } else {
                        piece_char.to_ascii_lowercase().align_to_center(7).black_pieces()
                    }
                },
                2 => {
                    if piece == Piece::NONE || piece == Piece::KING {
                        return String::new().align_to_center(7);
                    }

                    let mut board_cpy = *board;
                    board_cpy.remove_piece_on_square(square, piece, side);

                    if board_cpy.is_square_attacked(board.king_square(board.side().flipped()), board.side().flipped()) {
                        return "PIN".align_to_center(7).gray();
                    }

                    let modified_eval = board_evals[usize::from(square)];
                    let diff = current_eval - modified_eval;
                    let sign = if diff > 0 { "+" } else { "-" };
                    heat_color(format!("{}{}", sign, diff.abs()).align_to_center(7).as_str(), diff as f32 / 100.0, -20.0, 20.0)
                },
                _ => unreachable!()
            }
        };

        for row_idx in 0..3 {
            for temp_file in 0..8 {
                let square_rank = if board.side() == Side::WHITE { 7 - rank } else { rank };
                let square_file = if board.side() == Side::WHITE { temp_file } else { 7 - temp_file };
                print!("{}{}", "|".primary((rank * 4 + row_idx + 1) as f32 / 32.0), square_data(row_idx, square_rank, square_file));
                if temp_file == 7 {
                    println!("{}   {}", 
                        "|".primary((rank * 4 + row_idx + 1) as f32 / 32.0),
                        info[(rank * 4 + row_idx + 1) as usize]
                    )
                }
            }
        }
    }

    println!()
}