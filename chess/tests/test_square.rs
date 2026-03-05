use chess::{Bitboard, Square};

#[test]
fn shift_left() {
    assert_eq!(Square::from(32) << 3, Square::from(35));
    assert_eq!(
        Square::from(32) << 3,
        Bitboard::from(Square::from(32)).shift_left(3).ls1b_square()
    );
}

#[test]
fn shift_right() {
    assert_eq!(Square::from(32) >> 3, Square::from(29));
    assert_eq!(
        Square::from(32) >> 3,
        Bitboard::from(Square::from(32))
            .shift_right(3)
            .ls1b_square()
    );
}

#[test]
fn from_string() {
    assert_eq!(Square::from("h8"), Square::H8);
    assert_eq!(Square::from("b2".to_string()), Square::B2);
}

#[test]
fn to_string() {
    assert_eq!(String::from(Square::H8), "h8");
    assert_eq!(String::from(Square::B2), "b2");
    assert_eq!(String::from(Square::NULL), "NULL");
}
