use std::{fmt::Display, ops::Add};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeIndex(u32);
impl NodeIndex {
    pub const NULL: Self = Self(u32::MAX);

    pub fn new(half: u32, index: u32) -> Self {
        Self((half << 31) | (index & 0x7FFFFFFF))
    }

    pub fn is_null(&self) -> bool {
        *self == Self::NULL
    }

    pub fn half(&self) -> u32 {
        self.0 >> 31
    }

    pub fn idx(&self) -> u32 {
        self.0 & 0x7FFFFFFF
    }
}

impl From<u32> for NodeIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<NodeIndex> for u32 {
    fn from(value: NodeIndex) -> Self {
        value.0
    }
}

impl Add<usize> for NodeIndex {
    type Output = NodeIndex;

    fn add(self, rhs: usize) -> Self::Output {
        NodeIndex::new(self.half(), self.idx() + rhs as u32)
    }
}

impl Display for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = if self.is_null() {
            String::from("NULL")
        } else {
            format!("({}, {})", self.half(), self.idx())
        };

        write!(f, "{}", result)
    }
}