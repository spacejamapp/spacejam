//! Jastime IR

use std::collections::BTreeMap;
pub use {
    block::{Block, Control},
    func::{Export, Function},
};

mod block;
mod func;

/// Jastime IR
pub struct IR {
    /// The exports of the program
    pub exports: BTreeMap<u64, Vec<u64>>,

    /// The functions of the program
    pub funcs: BTreeMap<u64, Function>,
}

impl IR {
    /// Get an export from entry program counter
    pub fn exports(_entry: u64) -> Vec<Export> {
        Vec::new()
    }
}
