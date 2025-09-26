use chess::{ChessBoard, ChessPosition, Move, MoveFlag, Square, FEN};
use engine::{NoReport, SearchEngine, SearchLimits};

#[test]
fn three_fold() { 
    let mut search_engine = SearchEngine::new();

    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("7k/5pp1/rp1p2p1/q1p5/8/2Q5/6PP/7K w - - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::C3, Square::H3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::H8, Square::G8, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::H3, Square::C8, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::G8, Square::H7, MoveFlag::QUIET_MOVE), &mask);

    search_engine.set_position(&position, 0);
    
    let mut limits = SearchLimits::default();
    limits.set_iters(Some(20000));

    search_engine.search::<NoReport>(&limits);

    let best_move = search_engine.tree().get_best_pv(0, 0.5).first_move();
    assert_eq!(best_move, Move::from_squares(Square::C8, Square::H3, MoveFlag::QUIET_MOVE))
}

#[test]
fn fifty_mr() { 
    let mut search_engine = SearchEngine::new();

    let position = ChessPosition::from(ChessBoard::from(&FEN::from("1r5k/2q1q3/3q1q2/4q1q1/5q2/8/1P6/KR6 w - - 98 100")));

    search_engine.set_position(&position, 0);
    
    let mut limits = SearchLimits::default();
    limits.set_iters(Some(2000));

    search_engine.search::<NoReport>(&limits);

    let draw_distance = 0.5 - search_engine.tree().get_best_pv(0, 0.5).score().single();
    assert!(draw_distance.abs() < 0.1)
}

#[test]
fn fifty_mr_mate() { 
    let mut search_engine = SearchEngine::new();

    let position = ChessPosition::from(ChessBoard::from(&FEN::from("1r5k/2q1q3/3q1q2/4q1q1/5q2/8/1P6/KR6 b - - 99 100")));

    search_engine.set_position(&position, 0);
    
    let mut limits = SearchLimits::default();
    limits.set_iters(Some(2000));

    search_engine.search::<NoReport>(&limits);

    let draw_distance = 0.5 - search_engine.tree().get_best_pv(0, 0.5).score().single();
    assert!(draw_distance.abs() > 0.4)
}