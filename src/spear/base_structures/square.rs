use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, BitXor},
};

use super::bitboard::Bitboard;

#[derive(Copy, Clone, PartialEq, Hash)]
pub struct Square(u8);

impl Default for Square {
    fn default() -> Self {
        Square::NULL
    }
}

impl Square {
    pub const A1: Self = Self(0);
    pub const B1: Self = Self(1);
    pub const C1: Self = Self(2);
    pub const D1: Self = Self(3);
    pub const E1: Self = Self(4);
    pub const F1: Self = Self(5);
    pub const G1: Self = Self(6);
    pub const H1: Self = Self(7);
    pub const A2: Self = Self(8);
    pub const B2: Self = Self(9);
    pub const C2: Self = Self(10);
    pub const D2: Self = Self(11);
    pub const E2: Self = Self(12);
    pub const F2: Self = Self(13);
    pub const G2: Self = Self(14);
    pub const H2: Self = Self(15);
    pub const A3: Self = Self(16);
    pub const B3: Self = Self(17);
    pub const C3: Self = Self(18);
    pub const D3: Self = Self(19);
    pub const E3: Self = Self(20);
    pub const F3: Self = Self(21);
    pub const G3: Self = Self(22);
    pub const H3: Self = Self(23);
    pub const A4: Self = Self(24);
    pub const B4: Self = Self(25);
    pub const C4: Self = Self(26);
    pub const D4: Self = Self(27);
    pub const E4: Self = Self(28);
    pub const F4: Self = Self(29);
    pub const G4: Self = Self(30);
    pub const H4: Self = Self(31);
    pub const A5: Self = Self(32);
    pub const B5: Self = Self(33);
    pub const C5: Self = Self(34);
    pub const D5: Self = Self(35);
    pub const E5: Self = Self(36);
    pub const F5: Self = Self(37);
    pub const G5: Self = Self(38);
    pub const H5: Self = Self(39);
    pub const A6: Self = Self(40);
    pub const B6: Self = Self(41);
    pub const C6: Self = Self(42);
    pub const D6: Self = Self(43);
    pub const E6: Self = Self(44);
    pub const F6: Self = Self(45);
    pub const G6: Self = Self(46);
    pub const H6: Self = Self(47);
    pub const A7: Self = Self(48);
    pub const B7: Self = Self(49);
    pub const C7: Self = Self(50);
    pub const D7: Self = Self(51);
    pub const E7: Self = Self(52);
    pub const F7: Self = Self(53);
    pub const G7: Self = Self(54);
    pub const H7: Self = Self(55);
    pub const A8: Self = Self(56);
    pub const B8: Self = Self(57);
    pub const C8: Self = Self(58);
    pub const D8: Self = Self(59);
    pub const E8: Self = Self(60);
    pub const F8: Self = Self(61);
    pub const G8: Self = Self(62);
    pub const H8: Self = Self(63);
    pub const NULL: Self = Self(64);

    #[inline]
    pub const fn from_raw(value: u8) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn from_coords(rank: u8, file: u8) -> Self {
        Self(rank * 8 + file)
    }

    #[inline]
    pub const fn get_raw(&self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn get_rank(&self) -> u8 {
        self.get_raw() / 8
    }

    #[inline]
    pub const fn get_file(&self) -> u8 {
        self.get_raw() % 8
    }

    #[inline]
    pub const fn get_bit(&self) -> Bitboard {
        Bitboard::from_raw(1u64 << self.get_raw())
    }

    #[inline]
    pub const fn equals(&self, rhs: Square) -> bool {
        self.get_raw() == rhs.get_raw()
    }

    #[inline]
    pub const fn flip(&self) -> Self {
        Self::from_raw(self.get_raw() ^ 56)
    }

    #[inline]
    pub fn shift_left(&self, shift: u32) -> Self {
        (self.get_bit() << shift).ls1b_square()
    }

    #[inline]
    pub fn shift_right(&self, shift: u32) -> Self {
        (self.get_bit() >> shift).ls1b_square()
    }

    pub fn from_string(square: &str) -> Square {
        let signatures: Vec<char> = square.chars().collect();
        let file = signatures[0] as u8 - 'a' as u8;
        let rank = signatures[1].to_string().parse::<u8>().unwrap() - 1;
        Square::from_coords(rank, file)
    }
}
impl From<Square> for u8 {
    fn from(square: Square) -> Self {
        square.get_raw()
    }
}

impl From<Square> for usize {
    fn from(square: Square) -> Self {
        square.get_raw() as usize
    }
}

impl Add<u8> for Square {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u8) -> Self::Output {
        Self(self.get_raw() + rhs)
    }
}
impl Add<Square> for u8 {
    type Output = Square;

    #[inline]
    fn add(self, rhs: Square) -> Self::Output {
        Square(self + rhs.get_raw())
    }
}
impl BitXor<u8> for Square {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: u8) -> Self::Output {
        Self::from_raw(self.get_raw() ^ rhs)
    }
}

impl Display for Square {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        if *self == Square::NULL {
            return write!(formatter, "NULL");
        }

        let file = self.get_raw() % 8;
        let rank = ((self.get_raw() as f32) / 8_f32).floor() as u8 + 1;
        write!(formatter, "{}{}", ('a' as u8 + file) as char, rank)
    }
}
