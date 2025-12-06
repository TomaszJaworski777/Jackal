use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Piece(u8);
impl Piece {
    pub const PAWN: Self = Self(0);
    pub const KNIGHT: Self = Self(1);
    pub const BISHOP: Self = Self(2);
    pub const ROOK: Self = Self(3);
    pub const QUEEN: Self = Self(4);
    pub const KING: Self = Self(5);
    pub const NONE: Self = Self(u8::MAX);

    pub const fn value(&self) -> usize {
        self.0 as usize
    }
}

impl Default for Piece {
    fn default() -> Self {
        Self::NONE
    }
}

impl From<u8> for Piece {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<usize> for Piece {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u8)
    }
}

impl From<Piece> for u8 {
    #[inline]
    fn from(value: Piece) -> Self {
        value.0
    }
}

impl From<Piece> for usize {
    #[inline]
    fn from(value: Piece) -> Self {
        value.0 as usize
    }
}

impl From<Piece> for char {
    fn from(piece: Piece) -> Self {
        match piece {
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

impl Display for Piece {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", char::from(*self))
    }
}
