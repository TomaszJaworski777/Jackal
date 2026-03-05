use chess::{Bitboard, Square};

#[test]
fn shift_left() {
    assert_eq!(Bitboard::from(Square::A5) << 3, Bitboard::from(Square::D5));
}

#[test]
fn shift_right() {
    assert_eq!(Bitboard::from(Square::H5) >> 3, Bitboard::from(Square::E5));
}
