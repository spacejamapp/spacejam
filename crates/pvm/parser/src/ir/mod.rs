//! Jastime IR

use std::collections::BTreeMap;
pub use {
    block::{Block, Control},
    func::{Export, Function, FunctionRef},
};

mod block;
mod func;

/// Jastime IR
#[derive(Debug, Clone, Default)]
pub struct IR {
    /// The exports of the program
    pub exports: BTreeMap<u64, Vec<u64>>,

    /// The functions of the program
    pub funcs: BTreeMap<u64, FunctionRef>,

    /// The basic blocks of the program
    pub blocks: BTreeMap<u64, Block>,
}

impl IR {
    /// Get an export from entry program counter
    pub fn export(&self, _entry: u64) -> Option<Export> {
        None
    }

    /// Get a function from program counter
    ///
    /// or mb we just need a interfaces like functions?
    pub fn function(&self, _pc: u64) -> Option<Function> {
        None
    }
}
