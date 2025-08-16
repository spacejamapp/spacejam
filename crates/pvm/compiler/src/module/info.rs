//! Module execution info

use crate::module::Memory;

/// Result of executing a compiled module
#[derive(Debug, Clone)]
pub struct Info {
    /// Final register values
    pub registers: [u64; crate::constants::PVM_REGISTER_COUNT],
    /// Final program counter
    pub pc: u64,
    /// Final memory state
    pub memory: Memory,
}
