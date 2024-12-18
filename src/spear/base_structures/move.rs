use std::fmt::{Display, Formatter, Result};

use crate::spear::{Piece, Square};

use super::move_flags::MoveFlag;

#[derive(Copy, Clone, PartialEq, Default)]
//16 bit move
//0..5 -> from square
//6..9 -> flag
//10..15 -> to square
pub struct Move(u16);
impl Move {
    pub const NULL: Self = Self(0);

    #[inline]
    pub fn from_raw(value: u16) -> Self {
        Self(value)
    }

    #[inline]
    pub fn from_squares(from_square: Square, to_square: Square, flag: u16) -> Self {
        Self::from_raw(
            (to_square.get_raw() as u16) << 10 | flag << 6 | from_square.get_raw() as u16,
        )
    }

    #[inline]
    pub fn get_raw(&self) -> u16 {
        self.0
    }

    #[inline]
    pub fn get_from_square(&self) -> Square {
        Square::from_raw((self.0 & 63) as u8)
    }

    #[inline]
    pub fn get_to_square(&self) -> Square {
        Square::from_raw((self.0 >> 10) as u8)
    }

    #[inline]
    pub fn get_flag(&self) -> u16 {
        (self.0 >> 6) & 15
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        self.get_flag() & MoveFlag::CAPTURE > 0
    }

    #[inline]
    pub fn is_en_passant(&self) -> bool {
        self.get_flag() == MoveFlag::EN_PASSANT
    }

    #[inline]
    pub fn is_promotion(&self) -> bool {
        self.get_flag() & MoveFlag::KNIGHT_PROMOTION > 0
    }

    #[inline]
    pub fn get_promotion_piece(&self) -> Piece {
        Piece::from_raw(((self.get_flag() & 3) + 1) as u8)
    }
}

impl Display for Move {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(
            formatter,
            "{}{}{}",
            self.get_from_square(),
            self.get_to_square(),
            if self.is_promotion() {
                ["n", "b", "r", "q"][(self.get_promotion_piece().get_raw() - 1) as usize]
            } else {
                ""
            }
        )
    }
}
