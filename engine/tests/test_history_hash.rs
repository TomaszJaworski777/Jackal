use chess::{ChessBoard, ChessPosition, Move, MoveFlag, Square, FEN};

#[test]
fn validate_hash() {
    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::G1, Square::F3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::B8, Square::C6, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::B1, Square::C3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::G8, Square::F6, MoveFlag::QUIET_MOVE), &mask);

    let hash = position.history().hash();

    assert_eq!(hash >> 64, u64::from(position.board().hash()) as u128);
    assert_eq!(hash & 0b1111111, position.history().len() as u128);
}

#[test]
fn reversable_transposition() { 
    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::G1, Square::F3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::B8, Square::C6, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::B1, Square::C3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::G8, Square::F6, MoveFlag::QUIET_MOVE), &mask);

    let hash_a = position.history().hash();

    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::B1, Square::C3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::G8, Square::F6, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::G1, Square::F3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::B8, Square::C6, MoveFlag::QUIET_MOVE), &mask);

    let hash_b = position.history().hash();

    assert_ne!(hash_a, hash_b);

    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("rnbqkbnr/pppp1ppp/4p3/8/2P5/3P4/PP2PPPP/RNBQKBNR w KQkq - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::C1, Square::E3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::E8, Square::E7, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::D1, Square::A4, MoveFlag::QUIET_MOVE), &mask);

    let hash_a = position.history().hash();

    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("rnbqkbnr/pppp1ppp/4p3/8/2P5/3P4/PP2PPPP/RNBQKBNR w KQkq - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::D1, Square::A4, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::E8, Square::E7, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::C1, Square::E3, MoveFlag::QUIET_MOVE), &mask);

    let hash_b = position.history().hash();

    assert_ne!(hash_a, hash_b);
}

#[test]
fn irreversable_transposition() { 
    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::G1, Square::F3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::B8, Square::C6, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::B1, Square::C3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::G8, Square::F6, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::D2, Square::D4, MoveFlag::DOUBLE_PUSH), &mask);
    position.make_move(Move::from_squares(Square::D7, Square::D5, MoveFlag::DOUBLE_PUSH), &mask);

    let hash_a = position.history().hash();

    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::G1, Square::F3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::B8, Square::C6, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::B1, Square::C3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::G8, Square::F6, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::D2, Square::D4, MoveFlag::DOUBLE_PUSH), &mask);
    position.make_move(Move::from_squares(Square::D7, Square::D5, MoveFlag::DOUBLE_PUSH), &mask);

    let hash_b = position.history().hash();

    assert_eq!(hash_a, hash_b);

    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("rnbqkbnr/ppp1pppp/8/8/3p4/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::E2, Square::E4, MoveFlag::DOUBLE_PUSH), &mask);
    position.make_move(Move::from_squares(Square::D4, Square::E3, MoveFlag::EN_PASSANT), &mask);

    let hash_a = position.history().hash();

    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from("rnbqkbnr/ppp1pppp/8/8/3p4/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")));
    let mask = position.board().castle_rights().get_castle_mask();
    position.make_move(Move::from_squares(Square::E2, Square::E3, MoveFlag::QUIET_MOVE), &mask);
    position.make_move(Move::from_squares(Square::D4, Square::E3, MoveFlag::CAPTURE), &mask);

    let hash_b = position.history().hash();

    assert_eq!(hash_a, hash_b);
}