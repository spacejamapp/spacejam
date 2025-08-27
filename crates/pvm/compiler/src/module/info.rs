//! Module execution info
//!
//! TODO: merge with pvmi into pvm package

use pvm::Reason;

/// Result of executing a compiled module
#[derive(Debug, Clone)]
pub struct Info {
    /// Final register values
    pub registers: [u64; pvm::REGISTER_COUNT],
    /// Final program counter
    pub pc: u64,
    /// Final gas
    pub gas: u64,
    /// Final memory state
    pub memory: pvm::Memory,

    /// The exit reason
    pub reason: Reason,
}
