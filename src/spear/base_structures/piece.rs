use std::fmt::{Display, Formatter, Result};

#[derive(Copy, Clone, PartialEq)]
pub struct Piece(u8);
impl Piece {
    pub const PAWN: Self = Self(0);
    pub const KNIGHT: Self = Self(1);
    pub const BISHOP: Self = Self(2);
    pub const ROOK: Self = Self(3);
    pub const QUEEN: Self = Self(4);
    pub const KING: Self = Self(5);
    pub const NONE: Self = Self(u8::MAX);

    #[inline]
    pub const fn from_raw(value: u8) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn get_raw(&self) -> u8 {
        self.0
    }

    pub fn to_char(&self) -> char {
        match *self {
            Piece::PAWN => 'p',
            Piece::KNIGHT => 'n',
            Piece::BISHOP => 'b',
            Piece::ROOK => 'r',
            Piece::QUEEN => 'q',
            Piece::KING => 'k',
            _ => ' ',
        }
    }
}

impl From<Piece> for u8 {
    fn from(piece: Piece) -> Self {
        piece.get_raw()
    }
}

impl From<Piece> for usize {
    fn from(piece: Piece) -> Self {
        piece.get_raw() as usize
    }
}

impl Display for Piece {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self.to_char())
    }
}
