//! Memory abstraction for the VM

use anyhow::Result;

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

    /// Set the heap pointer.
    fn set_heap_ptr(&mut self, heap_ptr: u32);
}

impl MemoryLike for parser::Memory {
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

    fn set_heap_ptr(&mut self, heap_ptr: u32) {
        self.heap_ptr = heap_ptr;
    }
}

impl MemoryLike for &mut parser::Memory {
    fn read(&self, addr: u32, len: u32) -> Result<Vec<u8>> {
        self.read_bytes(addr, len)
    }

    fn write(&mut self, addr: u32, bytes: &[u8]) -> Result<()> {
        self.write_bytes(addr, bytes)
    }

    fn allocate(&mut self, start: u32, count: u32) -> Result<()> {
        parser::Memory::allocate(self, start, count)
    }

    fn heap_ptr(&self) -> u32 {
        self.heap_ptr
    }

    fn set_heap_ptr(&mut self, heap_ptr: u32) {
        self.heap_ptr = heap_ptr;
    }
}
