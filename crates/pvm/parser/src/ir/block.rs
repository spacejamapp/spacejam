//! Code block in a function

use crate::Instruction;
use core::ops::Range;
use std::collections::BTreeSet;

/// A code block in a function
#[derive(Debug, Clone, Default)]
pub struct Block {
    /// The range of the block
    pub range: Range<u64>,

    /// The instructions in the block
    pub code: Vec<Instruction>,

    /// The input registers of this block
    pub input: BTreeSet<u8>,

    /// The output registers of this block
    pub output: BTreeSet<u8>,

    /// The termination control flow info
    pub control: Control,
}

impl Block {
    /// Get the reachable program counter
    pub fn reachable(&self) -> u64 {
        match self.control {
            Control::Internal => self.range.end,
            Control::External(pc) => pc,
        }
    }
}

/// Control flow info
#[derive(Debug, Clone, Default)]
pub enum Control {
    /// Internal control flow
    #[default]
    Internal,

    /// External control flow
    External(u64),
}
