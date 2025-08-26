//! Runtime context for block execution

/// Runtime context for block execution
#[derive(Debug, Clone)]
pub struct Context {
    pub registers: [u64; pvm::REGISTER_COUNT],
    pub pc: u64,
    pub gas: u64,
}

impl Context {
    /// Create new context
    pub fn new(regs: [u64; pvm::REGISTER_COUNT], pc: u64) -> Self {
        Self {
            registers: regs,
            pc,
            gas: 0,
        }
    }
}
