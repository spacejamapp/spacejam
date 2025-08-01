//! Execution context

/// Execution context passed to compiled functions
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Context {
    /// PVM registers
    pub registers: [u64; 13],
    /// Program counter
    pub pc: u64,
}
