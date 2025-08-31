//! Memory management for PVM programs
//!
//! ## Linux
//! The virtual memory has verified on linux, works perfectly.
//!
//! ## macOS
//!
//! since macOS doesn't support virtual memory larger than 2.5GB thus we use
//! a hybrid approach to implement the memory management on macOS.
//!
//! - re-mapping allocated memory address to the head
//! - use a sperated heap track the heap area

pub use {hybrid::Memory as HybridMemory, hybrid::Memory, mmap::Memory as MmapMemory};

/* #[cfg(target_os = "linux")]
pub use mmap::Memory;

#[cfg(target_os = "macos")]
pub use hybrid::Memory; */

mod hybrid;
mod mmap;
