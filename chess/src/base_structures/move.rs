use crate::{
    base_structures::{Piece, Side},
    Square,
};

pub struct MoveFlag;
impl MoveFlag {
    pub const QUIET_MOVE: u16 = 0b0000 << 6;
    pub const DOUBLE_PUSH: u16 = 0b0001 << 6;
    pub const KING_SIDE_CASTLE: u16 = 0b0010 << 6;
    pub const QUEEN_SIDE_CASTLE: u16 = 0b0011 << 6;
    pub const CAPTURE: u16 = 0b0100 << 6;
    pub const EN_PASSANT: u16 = 0b0101 << 6;
    pub const KNIGHT_PROMOTION: u16 = 0b1000 << 6;
    pub const BISHOP_PROMOTION: u16 = 0b1001 << 6;
    pub const ROOK_PROMOTION: u16 = 0b1010 << 6;
    pub const QUEEN_PROMOTION: u16 = 0b1011 << 6;
    pub const KNIGHT_PROMOTION_CAPTURE: u16 = 0b1100 << 6;
    pub const BISHOP_PROMOTION_CAPTURE: u16 = 0b1101 << 6;
    pub const ROOK_PROMOTION_CAPTURE: u16 = 0b1110 << 6;
    pub const QUEEN_PROMOTION_CAPTURE: u16 = 0b1111 << 6;
}

//16 bit move
//0..5 -> from square
//6..9 -> flag
//10..15 -> to square

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Move(u16);
impl Move {
    pub const NULL: Self = Self(0);

    #[inline]
    pub fn from_squares(from_square: Square, to_square: Square, flag: u16) -> Self {
        Self(u16::from(to_square) << 10 | flag | u16::from(from_square))
    }

    #[inline]
    pub fn from_square(&self) -> Square {
        Square::from((self.0 & 63) as u8)
    }

    #[inline]
    pub fn to_square(&self) -> Square {
        Square::from((self.0 >> 10) as u8)
    }

    #[inline]
    pub fn flag(&self) -> u16 {
        self.0 & (15 << 6)
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        self.0 & MoveFlag::CAPTURE != 0
    }

    #[inline]
    pub fn is_en_passant(&self) -> bool {
        self.flag() == MoveFlag::EN_PASSANT
    }

    #[inline]
    pub fn is_castle(&self) -> bool {
        let flag = self.flag();
        flag == MoveFlag::KING_SIDE_CASTLE || flag == MoveFlag::QUEEN_SIDE_CASTLE
    }

    #[inline]
    pub fn is_promotion(&self) -> bool {
        self.0 & MoveFlag::KNIGHT_PROMOTION != 0
    }

    #[inline]
    pub fn promotion_piece(&self) -> Piece {
        Piece::from((((self.flag() >> 6) & 3) + 1) as u8)
    }

    pub fn to_string(&self, chess960: bool) -> String {
        if !chess960 && self.is_castle() {
            let side = if u8::from(self.from_square()) < 32 {
                Side::WHITE
            } else {
                Side::BLACK
            };
            let destination_square = if self.flag() == MoveFlag::QUEEN_SIDE_CASTLE {
                Square::C1
            } else {
                Square::G1
            } + 56 * u8::from(side);
            return format!("{}{}", self.from_square(), destination_square);
        }

        format!(
            "{}{}{}",
            self.from_square(),
            self.to_square(),
            if self.is_promotion() {
                ["n", "b", "r", "q"][(u8::from(self.promotion_piece()) - 1) as usize]
            } else {
                ""
            }
        )
    }
}

impl From<Move> for u16 {
    #[inline]
    fn from(value: Move) -> Self {
        value.0
    }
}

impl From<u16> for Move {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}
