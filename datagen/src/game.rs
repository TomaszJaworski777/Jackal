use chess::{ChessPosition, Move, MoveFlag, Side, Square, FEN};
use engine::{NoReport, SearchEngine, SearchLimits};
use montyformat::{chess::{Castling, Position}, MontyFormat, SearchData};
use rand::Rng;

pub fn play_game(engine: &mut SearchEngine, position: &mut ChessPosition, limits: &SearchLimits, avg_iters: &mut u64) -> MontyFormat {
    let castle_mask = position.board().castle_rights().get_castle_mask();

    let mut monty_castling = Castling::default();
    let monty_position = Position::parse_fen(FEN::from(position.board()).to_string().as_str(), &mut monty_castling);
    let mut game_data = MontyFormat::new(monty_position, monty_castling);

    let mut temperature = 0.77;

    let mut iter_sum = 0u64;

    loop {
        engine.tree().clear();
        engine.set_position(position, 10);

        let stats = engine.search::<NoReport>(limits);
        iter_sum += stats.iterations();
        
        let mut moves = Vec::new();
        let mut best_move = Move::NULL;
        let mut best_monty_move = montyformat::chess::Move::NULL;
        let mut best_score = f64::MIN;

        let total = if temperature >= 0.2 {
            let mut result = 0.0;

            engine.tree().root_node().map_children(|child_idx| { 
                let node = &engine.tree()[child_idx];
                result += (node.visits() as f64).powf(1.0 / temperature)
            } );            

            result
        } else {
            1.0
        }; 

        let mut sum = 0.0;
        let threshold = rand::rng().random_range(0.0..=1.0);
        engine.tree().root_node().map_children(|child_idx| {
            let node = &engine.tree()[child_idx];

            let mv = node.mv();

            let monty_move = move_to_monty(mv, engine);

            moves.push((monty_move, node.visits()));

            let score = node.score().single_with_score(engine.options().draw_score() as f64 / 100.0);

            if temperature < 0.2 && score > best_score {
                best_score = score;
                best_move = mv;
                best_monty_move = monty_move;

                return;
            }

            if best_move != Move::NULL {
                return;
            }

            sum += (node.visits() as f64).powf(1.0 / temperature);

            if sum / total >= threshold {
                best_score = score;
                best_move = mv;
                best_monty_move = monty_move;
            }
        });

        moves.sort_by_key(|(mv, _)| u16::from(*mv));

        game_data.push(SearchData {
            best_move: best_monty_move,
            score: best_score as f32,
            visit_distribution: Some(moves)
        });

        position.make_move(best_move, &castle_mask);
        temperature *= 0.91;

        let mut no_legal_moves = true;
        position.board().map_legal_moves(|_| no_legal_moves = false );

        if no_legal_moves {
            game_data.result = if position.board().is_in_check() {
                if position.board().side() == Side::WHITE {
                    0.0
                } else {
                    1.0
                }
            } else {
                0.5
            };

            break;
        }

        if position.board().half_moves() >= 100 || position.history().get_repetitions(position.board().hash()) >= 3 {
           game_data.result = 0.5; 
           break;
        }
    }

    *avg_iters = iter_sum / game_data.moves.len() as u64;

    game_data
}

fn move_to_monty(mv: Move, engine: &SearchEngine) -> montyformat::chess::Move {
    let from = u8::from(mv.get_from_square()) as u16;
    let mut to = u8::from(mv.get_to_square()) as u16;

    if !engine.options().chess960() && mv.is_castle() {
        let side = usize::from(engine.root_position().board().side());

        if mv.get_flag() == MoveFlag::KING_SIDE_CASTLE {
           to = u8::from(Square::G1) as u16 ^ (56 * side) as u16; 
        } else {
            to = u8::from(Square::C1) as u16 ^ (56 * side) as u16;  
        }
    }

    montyformat::chess::Move::new(from, to, mv.get_flag() >> 6)
}