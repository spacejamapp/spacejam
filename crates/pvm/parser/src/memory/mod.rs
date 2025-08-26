//! Memory implementation

use anyhow::Result;
pub use {btree::Memory, info::MemoryInfo};

mod btree;
mod info;

/// A trait for memory-like objects.
pub trait MemoryLike {
    /// Read bytes from the memory.
    fn read(&self, addr: u32, len: u32) -> Result<Vec<u8>>;

    /// Write bytes to the memory.
    fn write(&mut self, addr: u32, bytes: &[u8]) -> Result<()>;

    /// Allocate a range of memory.
    fn allocate(&mut self, start: u32, count: u32) -> Result<()>;

    /// Get the heap pointer.
    fn heap_ptr(&self) -> u32;
}

impl MemoryLike for Memory {
    fn read(&self, addr: u32, len: u32) -> Result<Vec<u8>> {
        self.read_bytes(addr, len)
    }

    fn write(&mut self, addr: u32, bytes: &[u8]) -> Result<()> {
        self.write_bytes(addr, bytes)
    }

    fn allocate(&mut self, start: u32, count: u32) -> Result<()> {
        self.allocate(start, count)
    }

    fn heap_ptr(&self) -> u32 {
        self.heap_ptr
    }
}
