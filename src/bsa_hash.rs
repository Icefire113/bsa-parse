use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct BsaHash {
    low: u32,
    high: u32,
}

impl Display for BsaHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}{:08x}", self.low, self.high)
    }
}
