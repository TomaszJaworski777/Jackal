use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Copy, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct NodeIndex(u64);
impl NodeIndex {
    pub const NULL: NodeIndex = Self(0);

    #[inline]
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    #[inline]
    pub fn get_raw(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.get_raw() == 0
    }
}

impl Display for NodeIndex {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{:#x}", self.get_raw())
    }
}
