use chess::{Move, MoveFlag, Piece, Square};

#[test]
fn to_string() {
    assert_eq!(Move::NULL.to_string(false), "a1a1");
    assert_eq!(
        Move::from_squares(Square::A4, Square::B5, 0).to_string(false),
        "a4b5"
    );
    assert_eq!(
        Move::from_squares(Square::E7, Square::E8, MoveFlag::ROOK_PROMOTION).to_string(false),
        "e7e8r"
    );
    assert_eq!(
        Move::from_squares(Square::E1, Square::H1, MoveFlag::KING_SIDE_CASTLE).to_string(false),
        "e1g1"
    );
    assert_eq!(
        Move::from_squares(Square::E1, Square::A1, MoveFlag::QUEEN_SIDE_CASTLE).to_string(false),
        "e1c1"
    );
    assert_eq!(
        Move::from_squares(Square::E1, Square::H1, MoveFlag::KING_SIDE_CASTLE).to_string(true),
        "e1h1"
    );
    assert_eq!(
        Move::from_squares(Square::E1, Square::A1, MoveFlag::QUEEN_SIDE_CASTLE).to_string(true),
        "e1a1"
    );
}

#[test]
fn is_capture() {
    assert_eq!(Move::NULL.is_capture(), false);
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::EN_PASSANT).is_capture(),
        true
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION).is_capture(),
        false
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION_CAPTURE).is_capture(),
        true
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::CAPTURE).is_capture(),
        true
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::KING_SIDE_CASTLE).is_capture(),
        false
    );
}

#[test]
fn is_promotion() {
    assert_eq!(Move::NULL.is_promotion(), false);
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::EN_PASSANT).is_promotion(),
        false
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION).is_promotion(),
        true
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION_CAPTURE).is_promotion(),
        true
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::CAPTURE).is_promotion(),
        false
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::KING_SIDE_CASTLE).is_promotion(),
        false
    );
}

#[test]
fn promotion_piece() {
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::KNIGHT_PROMOTION)
            .get_promotion_piece(),
        Piece::KNIGHT
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::BISHOP_PROMOTION)
            .get_promotion_piece(),
        Piece::BISHOP
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION).get_promotion_piece(),
        Piece::ROOK
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::QUEEN_PROMOTION).get_promotion_piece(),
        Piece::QUEEN
    );

    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::KNIGHT_PROMOTION_CAPTURE)
            .get_promotion_piece(),
        Piece::KNIGHT
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::BISHOP_PROMOTION_CAPTURE)
            .get_promotion_piece(),
        Piece::BISHOP
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION_CAPTURE)
            .get_promotion_piece(),
        Piece::ROOK
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::QUEEN_PROMOTION_CAPTURE)
            .get_promotion_piece(),
        Piece::QUEEN
    );
}
