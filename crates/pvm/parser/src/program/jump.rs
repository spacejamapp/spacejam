//! The dynamic jump table.

use core::ops::Range;

/// The dynamic jump table.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JumpTable {
    /// The index of the jump table.
    pub index: Vec<u8>,

    /// The table.
    pub table: Vec<u8>,

    /// The range of the jump table.
    pub range: Range<usize>,
}
