//! Jastime function

use crate::ir::Block;
use core::ops::Range;
use std::collections::{BTreeMap, BTreeSet};

/// Jastime function
#[derive(Debug, Clone, Default)]
pub struct Function {
    /// The range of the function
    pub range: Range<u64>,

    /// The jump table of the function (index -> program counter)
    pub jump: BTreeMap<u32, u64>,

    /// The blocks in the function
    pub blocks: BTreeMap<u64, Block>,
}

/// Reference to a function
#[derive(Debug, Clone, Default)]
pub struct FunctionRef {
    /// The range of the function
    pub range: Range<u64>,

    /// The jump table of the function (index -> program counter)
    pub jump: BTreeMap<u32, u64>,

    /// The blocks in the function
    pub blocks: BTreeSet<u64>,
}

impl FunctionRef {
    /// Create a new function reference
    pub fn new(pc: u64) -> Self {
        let mut blocks = BTreeSet::new();
        blocks.insert(pc);

        Self {
            range: pc..pc,
            jump: BTreeMap::new(),
            blocks,
        }
    }
}

/// Export info
#[derive(Debug, Clone, Default)]
pub struct Export {
    /// The entry program counter of the function
    pub entry: u64,

    /// The function in the export
    pub funcs: Vec<u64>,
}
