use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct NodeIndex(i32);
impl NodeIndex {
    pub const NULL: NodeIndex = Self(-1);

    #[inline]
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    #[inline]
    pub fn get_raw(&self) -> i32 {
        self.0
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.get_raw() == -1
    }
}

impl Display for NodeIndex {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self.get_raw())
    }
}