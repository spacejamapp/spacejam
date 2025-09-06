//! Jastime function

use crate::ir::Block;
use core::ops::Range;
use std::collections::BTreeMap;

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
    pub blocks: BTreeMap<u64, u64>,
}

/// Export info
#[derive(Debug, Clone, Default)]
pub struct Export {
    /// The entry program counter of the function
    pub entry: u64,

    /// The function in the export
    pub funcs: Vec<u64>,
}
