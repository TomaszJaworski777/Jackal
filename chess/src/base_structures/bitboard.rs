use std::{
    fmt::{Display, Formatter, Result},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
        ShrAssign, Sub,
    },
};

use utils::Colors;

use super::square::Square;

#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct Bitboard(u64);
impl Bitboard {
    pub const RANK_1: Self = Self(0x00000000000000FF);
    pub const RANK_2: Self = Self(0x000000000000FF00);
    pub const RANK_3: Self = Self(0x0000000000FF0000);
    pub const RANK_4: Self = Self(0x00000000FF000000);
    pub const RANK_5: Self = Self(0x000000FF00000000);
    pub const RANK_6: Self = Self(0x0000FF0000000000);
    pub const RANK_7: Self = Self(0x00FF000000000000);
    pub const RANK_8: Self = Self(0xFF00000000000000);

    pub const FILE_A: Self = Self(0x0101010101010101);
    pub const FILE_B: Self = Self(0x0202020202020202);
    pub const FILE_C: Self = Self(0x0404040404040404);
    pub const FILE_D: Self = Self(0x0808080808080808);
    pub const FILE_E: Self = Self(0x1010101010101010);
    pub const FILE_F: Self = Self(0x2020202020202020);
    pub const FILE_G: Self = Self(0x4040404040404040);
    pub const FILE_H: Self = Self(0x8080808080808080);

    pub const FILES_AB: Self = Bitboard::FILE_A.or(Bitboard::FILE_B);
    pub const FILES_GH: Self = Bitboard::FILE_G.or(Bitboard::FILE_H);

    pub const FULL: Self = Self(0xFFFFFFFFFFFFFFFF);
    pub const EMPTY: Self = Self(0);

    #[inline]
    pub const fn from_value(value: u64) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn from_square(square: Square) -> Self {
        Self(1u64 << square.get_value())
    }

    #[inline]
    pub const fn get_value(&self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn pop_count(&self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub fn ls1b_square(&self) -> Square {
        Square::from(self.0.trailing_zeros() as u8)
    }

    #[inline]
    pub fn set_bit(&mut self, square: Square) {
        self.0 |= Bitboard::from(square);
    }

    #[inline]
    pub fn pop_bit(&mut self, square: Square) {
        self.0 &= !Bitboard::from(square);
    }

    #[inline]
    pub fn pop_ls1b_square(&mut self) -> Square {
        let square = self.ls1b_square();
        self.0 &= self.0 - 1;
        square
    }

    #[inline]
    pub const fn get_bit(&self, square: Square) -> bool {
        !self.and(Bitboard(1u64 << square.get_value())).is_empty()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn is_not_empty(&self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub const fn equals(&self, rhs: Bitboard) -> bool {
        self.0 == rhs.0
    }

    #[inline]
    pub const fn only_one_bit(&self) -> bool {
        !self.is_empty() && (self.0 & self.0.wrapping_sub(1)) == 0
    }

    #[inline]
    pub const fn multiple_one_bits(&self) -> bool {
        !self.is_empty() && (self.0 & self.0.wrapping_sub(1)) != 0
    }

    #[inline]
    pub fn mut_or(&mut self, rhs: Bitboard) {
        self.0 |= rhs.0;
    }

    #[inline]
    pub fn mut_and(&mut self, rhs: Bitboard) {
        self.0 &= rhs.0;
    }

    pub fn map<F: FnMut(Square)>(&self, mut method: F) {
        let mut bitboard_copy = *self;
        while bitboard_copy.is_not_empty() {
            method(bitboard_copy.pop_ls1b_square())
        }
    }

    #[inline]
    pub const fn and(&self, rhs: Bitboard) -> Self {
        Self(self.0 & rhs.0)
    }

    #[inline]
    pub const fn or(&self, rhs: Bitboard) -> Self {
        Self(self.0 | rhs.0)
    }

    #[inline]
    pub const fn xor(&self, rhs: Bitboard) -> Self {
        Self(self.0 ^ rhs.0)
    }

    #[inline]
    pub const fn inverse(&self) -> Self {
        Self(!self.0)
    }

    #[inline]
    pub const fn flip(&self) -> Self {
        Self(self.0.swap_bytes())
    }

    #[inline]
    pub const fn flip_mut(&mut self) {
        self.0 = self.0.swap_bytes()
    }

    #[inline]
    pub const fn include(&self, square: Square) -> Self {
        self.or(Bitboard(1u64 << square.get_value()))
    }

    #[inline]
    pub const fn exclude(&self, square: Square) -> Self {
        self.and(Bitboard(1u64 << square.get_value()).inverse())
    }

    #[inline]
    pub const fn shift_left(self, rhs: u32) -> Self {
        Self(self.0 << rhs)
    }

    #[inline]
    pub const fn shift_right(self, rhs: u32) -> Self {
        Self(self.0 >> rhs)
    }

    #[inline]
    pub const fn wrapping_mul(self, rhs: Bitboard) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }

    #[inline]
    pub const fn wrapping_sub(self, rhs: Bitboard) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }

    pub fn draw_bitboard(&self) {
        println!("{}", String::from(*self));
    }
}

impl From<u64> for Bitboard {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Bitboard> for u64 {
    #[inline]
    fn from(value: Bitboard) -> Self {
        value.0
    }
}

impl From<Square> for Bitboard {
    #[inline]
    fn from(value: Square) -> Self {
        Self(1u64 << u8::from(value))
    }
}

impl From<Bitboard> for String {
    #[inline]
    fn from(value: Bitboard) -> Self {
        let mut result = " -----------------\n".to_string();
        for rank in (0..8).rev() {
            result += "|";
            for file in 0..8 {
                let square = Square::from_coords(rank, file);
                result += if value.get_bit(square) {
                    " 1".green()
                } else {
                    " 0".red()
                }
                .to_string()
                .as_str();
            }
            result += " |\n";
        }
        result += " -----------------\n";
        result += &format!("  Bitboard: {}\n", u64::from(value));
        result
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAnd<u64> for Bitboard {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: u64) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl BitAnd<Bitboard> for u64 {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard(self & rhs.0)
    }
}

impl BitAndAssign<u64> for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: u64) {
        self.0 &= rhs;
    }
}

impl BitAndAssign<Bitboard> for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Bitboard) {
        self.0 &= rhs.0;
    }
}

impl BitAndAssign<Bitboard> for u64 {
    #[inline]
    fn bitand_assign(&mut self, rhs: Bitboard) {
        *self &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOr<u64> for Bitboard {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: u64) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl BitOr<Bitboard> for u64 {
    type Output = Bitboard;

    #[inline]
    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Bitboard(self | rhs.0)
    }
}

impl BitOrAssign<u64> for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 |= rhs;
    }
}

impl BitOrAssign<Bitboard> for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Bitboard) {
        self.0 |= rhs.0;
    }
}

impl BitOrAssign<Bitboard> for u64 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Bitboard) {
        *self |= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXor<u64> for Bitboard {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: u64) -> Self::Output {
        Self(self.0 ^ rhs)
    }
}

impl BitXor<Bitboard> for u64 {
    type Output = Bitboard;

    #[inline]
    fn bitxor(self, rhs: Bitboard) -> Self::Output {
        Bitboard(self ^ rhs.0)
    }
}

impl BitXorAssign<u64> for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u64) {
        self.0 ^= rhs;
    }
}

impl BitXorAssign<Bitboard> for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        self.0 ^= rhs.0;
    }
}

impl BitXorAssign<Bitboard> for u64 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        *self ^= rhs.0;
    }
}

impl Not for Bitboard {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Shl<u8> for Bitboard {
    type Output = Self;

    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl ShlAssign<u8> for Bitboard {
    #[inline]
    fn shl_assign(&mut self, rhs: u8) {
        self.0 <<= rhs;
    }
}

impl Shr<u8> for Bitboard {
    type Output = Self;

    #[inline]
    fn shr(self, rhs: u8) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl ShrAssign<u8> for Bitboard {
    #[inline]
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs;
    }
}

impl Sub<u64> for Bitboard {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Display for Bitboard {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        writeln!(formatter, "{}", String::from(*self))
    }
}
