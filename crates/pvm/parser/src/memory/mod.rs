//! Memory implementation

pub use btree::Memory;
use core::ops::Range;

mod btree;

/// Memory information
#[derive(Default, Clone, Debug)]
pub struct MemoryInfo {
    /// The read range.
    pub read: Range<u32>,

    /// The write range.
    pub write: Range<u32>,

    /// The heap range.
    pub heap: Range<u32>,

    /// The stack range.
    pub stack: Range<u32>,

    /// The args range.
    pub args: Range<u32>,
}
