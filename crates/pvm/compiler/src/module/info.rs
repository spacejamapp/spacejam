//! Module execution info

/// Result of executing a compiled module
#[derive(Debug, Clone)]
pub struct Info {
    /// Final register values
    pub registers: [u64; 13],
    /// Final program counter
    pub pc: u64,
}
