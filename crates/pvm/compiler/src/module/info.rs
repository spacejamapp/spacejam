//! Module execution info

use crate::module::Memory;

/// Result of executing a compiled module
#[derive(Debug, Clone)]
pub struct Info {
    /// Final register values
    pub registers: [u64; translator::constants::PVM_REGISTER_COUNT],
    /// Final program counter
    pub pc: u64,
    /// Final memory state
    pub memory: Memory,
}

/// Block execution result
#[derive(Debug, Clone)]
pub enum ExecResult {
    Continue,
    Jump(u64),
    Halt,
    Trap,
}
