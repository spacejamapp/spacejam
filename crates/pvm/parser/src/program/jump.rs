//! The dynamic jump table.

use crate::format::ISA;
use core::ops::Range;

/// The dynamic jump table.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JumpTable {
    /// The entry size of the jump table.
    pub entry_size: usize,

    /// The length of the jump table.
    pub len: usize,

    /// The table.
    pub table: Vec<u8>,

    /// The range of the jump table.
    pub range: Range<usize>,
}

impl JumpTable {
    /// Get the jump table entry at the given index.
    pub fn get(&self, index: usize) -> Option<usize> {
        if self.entry_size == 0 {
            return None;
        }

        self.table
            .windows(self.entry_size)
            .nth(index)
            .map(|w| u64::read(w).0 as usize)
    }
}
