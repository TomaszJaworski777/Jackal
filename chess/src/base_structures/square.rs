use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, BitXor, Shl, Shr},
};

#[derive(Debug, Copy, Clone, PartialEq)]
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
    pub const fn from_value(value: u8) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn from_coords(rank: u8, file: u8) -> Self {
        Self(rank * 8 + file)
    }

    #[inline]
    pub const fn get_value(&self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn get_rank(&self) -> u8 {
        self.0 / 8
    }

    #[inline]
    pub const fn get_file(&self) -> u8 {
        self.0 % 8
    }

    #[inline]
    pub const fn equals(&self, rhs: Square) -> bool {
        self.0 == rhs.0
    }

    #[inline]
    pub const fn flip(&self) -> Self {
        Self(self.0 ^ 56)
    }
}

impl From<u8> for Square {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Square> for u8 {
    #[inline]
    fn from(value: Square) -> Self {
        value.0
    }
}

impl From<Square> for u16 {
    #[inline]
    fn from(value: Square) -> Self {
        u16::from(value.0)
    }
}

impl From<Square> for usize {
    #[inline]
    fn from(value: Square) -> Self {
        value.0 as usize
    }
}

impl From<String> for Square {
    #[inline]
    fn from(value: String) -> Self {
        let signatures: Vec<char> = value.chars().collect();
        let file = signatures[0] as u8 - b'a';
        let rank = signatures[1].to_string().parse::<u8>().unwrap() - 1;
        Square::from_coords(rank, file)
    }
}

impl From<&str> for Square {
    #[inline]
    fn from(value: &str) -> Self {
        let signatures: Vec<char> = value.chars().collect();
        let file = signatures[0] as u8 - b'a';
        let rank = signatures[1].to_string().parse::<u8>().unwrap() - 1;
        Square::from_coords(rank, file)
    }
}

impl From<Square> for String {
    #[inline]
    fn from(value: Square) -> Self {
        if value == Square::NULL {
            return String::from("NULL");
        }

        format!(
            "{}{}",
            (b'a' + value.get_file()) as char,
            value.get_rank() + 1
        )
    }
}

impl Add<u8> for Square {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u8) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add<Square> for u8 {
    type Output = Square;

    #[inline]
    fn add(self, rhs: Square) -> Self::Output {
        Square(self + rhs.0)
    }
}

impl BitXor<u8> for Square {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: u8) -> Self::Output {
        Self(self.0 ^ rhs)
    }
}

impl Shl<u8> for Square {
    type Output = Self;

    #[inline]
    fn shl(self, value: u8) -> Self::Output {
        Self(self.0 + u8::from(value))
    }
}

impl Shr<u8> for Square {
    type Output = Self;

    #[inline]
    fn shr(self, value: u8) -> Self::Output {
        Self(self.0 - u8::from(value))
    }
}

impl Display for Square {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", String::from(*self))
    }
}
