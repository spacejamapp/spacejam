//! Memory management for PVM programs on macOS
//!
//! ## macOS
//!
//! since macOS doesn't support virtual memory larger than 2.5GB thus we use
//! a hybrid approach to implement the memory management on macOS.
//!
//! - re-mapping allocated memory address to the head
//! - use a sperated heap track the heap area

/// Hybrid memory management for PVM programs on macOS
///
/// the original PVM memory layout is as follows:
///
/// [ [ro data] [rw data] [heap] [stack] [args] ]
///
/// while in our hybrid approach, we re-map the allocated memory address to the head
/// and use a sperated heap to track the heap area.
///
/// [ [rw data] [ro data] [stack] [args] [heap] ]
#[derive(Debug, Clone)]
pub struct Memory {
    /// Base pointer
    base: *mut u8,

    /// Heap pointer
    heap: Vec<u8>,

    /// The offset between ro-data and stack previous the heap area.
    offset: u32,
}
