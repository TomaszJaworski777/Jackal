use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Side(u8);
impl Side {
    pub const WHITE: Self = Self(0);
    pub const BLACK: Self = Self(1);

    #[inline]
    pub const fn get_value(&self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn flipped(&self) -> Self {
        Self(1 - self.0)
    }

    #[inline]
    pub const fn flip(&mut self) {
        self.0 = 1 - self.0;
    }
}

impl From<bool> for Side {
    fn from(value: bool) -> Self {
        Self(u8::from(value))
    }
}

impl From<u8> for Side {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Side> for u8 {
    fn from(value: Side) -> Self {
        value.0
    }
}

impl From<Side> for usize {
    fn from(value: Side) -> Self {
        value.0 as usize
    }
}

impl From<Side> for String {
    fn from(value: Side) -> Self {
        if value == Side::WHITE {
            "White"
        } else {
            "Black"
        }
        .to_string()
    }
}

impl Display for Side {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", String::from(*self))
    }
}
