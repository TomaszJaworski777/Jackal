use chess::{ChessBoard, ChessPosition, Move, MoveFlag, Square, FEN};
use engine::{NoReport, SearchEngine, SearchLimits};

#[test]
fn mate_in_1() { 
    let mut search_engine = SearchEngine::new();

    let position = ChessPosition::from(ChessBoard::from(&FEN::from("1r5k/8/8/8/8/8/1P6/KR6 b - - 0 1")));

    search_engine.set_position(&position, 0);
    
    let mut limits = SearchLimits::default();
    limits.set_iters(Some(2000));

    search_engine.search::<NoReport>(&limits);

    let best_move = search_engine.tree().get_best_pv(0, search_engine.options().draw_score() as f64 / 100.0).first_move();
    assert_eq!(best_move, Move::from_squares(Square::B8, Square::A8, MoveFlag::QUIET_MOVE))
}

#[test]
fn mate_in_2() { 
    let mut search_engine = SearchEngine::new();

    let position = ChessPosition::from(ChessBoard::from(&FEN::from("r1b2k1r/ppp1bppp/8/1B1Q4/5q2/2P5/PPP2PPP/R3R1K1 w - - 1 1")));

    search_engine.set_position(&position, 0);
    
    let mut limits = SearchLimits::default();
    limits.set_iters(Some(125000));

    search_engine.search::<NoReport>(&limits);

    let best_move = search_engine.tree().get_best_pv(0, search_engine.options().draw_score() as f64 / 100.0).first_move();
    assert_eq!(best_move, Move::from_squares(Square::D5, Square::D8, MoveFlag::QUIET_MOVE))
}