//! Execution context

/// Execution context passed to compiled functions
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Context {
    /// PVM registers
    pub registers: [u64; 13],
    /// Program counter
    pub pc: u64,
    /// Memory state pointer (serialized memory data)
    pub memory_ptr: *mut u8,
    /// Memory data size
    pub memory_size: usize,
}
