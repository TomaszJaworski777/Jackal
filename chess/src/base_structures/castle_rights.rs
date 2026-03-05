use std::fmt::{Display, Formatter, Result};

use crate::Square;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct CastleRights {
    value: u8,
    rooks: [Square; 4],
    kings: [Square; 2],
}

impl CastleRights {
    pub const BLACK_KING: u8 = 0b0001;
    pub const BLACK_QUEEN: u8 = 0b0010;
    pub const WHITE_KING: u8 = 0b0100;
    pub const WHITE_QUEEN: u8 = 0b1000;
    pub const NULL: u8 = 0;

    pub fn create_base(rooks: [Square; 4], kings: [Square; 2]) -> Self {
        Self {
            value: 0,
            rooks,
            kings,
        }
    }

    #[inline]
    pub fn set_rights(&mut self, value: u8) {
        self.value = value
    }

    #[inline]
    pub fn has_right(&self, right_flag: u8) -> bool {
        self.value & right_flag != 0
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        self.value.trailing_zeros() as usize
    }

    #[inline]
    pub fn rook_square(&self, index: usize) -> Square {
        self.rooks[index]
    }

    #[inline]
    pub fn get_castle_mask(&self) -> [u8; 64] {
        let mut result = [0u8; 64];

        result[usize::from(self.kings[0])] = 0b1100;
        result[usize::from(self.kings[1])] = 0b0011;

        for idx in 0..4 {
            if self.rooks[idx] != Square::NULL {
                result[usize::from(self.rooks[idx])] = 0b1000 >> idx;
            }
        }

        result
    }
}

impl From<&CastleRights> for u8 {
    fn from(rights: &CastleRights) -> Self {
        rights.value
    }
}

impl From<&CastleRights> for usize {
    fn from(rights: &CastleRights) -> Self {
        rights.value as usize
    }
}

impl From<CastleRights> for String {
    fn from(value: CastleRights) -> Self {
        let mut result = String::new();
        if value.has_right(CastleRights::WHITE_KING) {
            result.push(char::from(b'A' + value.rooks[1].file()));
        }
        if value.has_right(CastleRights::WHITE_QUEEN) {
            result.push(char::from(b'A' + value.rooks[0].file()));
        }
        if value.has_right(CastleRights::BLACK_KING) {
            result.push(char::from(b'a' + value.rooks[3].file()));
        }
        if value.has_right(CastleRights::BLACK_QUEEN) {
            result.push(char::from(b'a' + value.rooks[2].file()));
        }
        if result.is_empty() {
            result.push('-');
        }

        result
    }
}

impl Display for CastleRights {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", String::from(*self))
    }
}
