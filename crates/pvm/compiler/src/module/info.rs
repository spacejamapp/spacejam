//! Module execution info

use super::memory::Memory;

/// Result of executing a compiled module
#[derive(Debug, Clone)]
pub struct Info {
    /// Final register values
    pub registers: [u64; 13],
    /// Final program counter
    pub pc: u64,
    /// Final memory state
    pub memory: Memory,
}
