use std::{fmt::{Display, Formatter, Result}, u32};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct NodeIndex(u32);
impl NodeIndex {
    pub const NULL: NodeIndex = Self(u32::MAX);

    #[inline]
    pub fn from_raw(value: u32) -> Self {
        Self(value)
    }

    #[inline]
    pub fn from_parts(index: u32, segment: u32) -> Self {
        Self((segment << 30) | (index & 0x3FFFFFFF))
    }

    #[inline]
    pub fn get_raw(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.0 & 0x3FFFFFFF
    }

    #[inline]
    pub fn segment(&self) -> usize {
        (self.0 >> 30) as usize
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        *self == Self::NULL
    }
}

impl Display for NodeIndex {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", if self.is_null() { "NULL".to_string() } else { format!("({}, {})", self.segment(), self.index()) })
    }
}