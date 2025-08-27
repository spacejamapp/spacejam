//! Runtime context for block execution

use crate::Memory;

/// Runtime context for block execution
#[derive(Debug)]
pub struct Context {
    pub registers: [u64; pvm::REGISTER_COUNT],
    pub pc: u64,
    pub gas: u64,
    pub memory: Memory,
}

impl Context {
    /// Create new context
    pub fn new(regs: [u64; pvm::REGISTER_COUNT], pc: u64, memory: Memory) -> Self {
        Self {
            registers: regs,
            pc,
            gas: 0,
            memory,
        }
    }
}
