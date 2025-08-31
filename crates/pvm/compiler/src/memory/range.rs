//! Memory management for PVM programs on macOS
//!
//! ## macOS
//!
//! since macOS doesn't support large virtual memory, thus we use
//! a range table to implement the memory management on macOS.
//!
//! - re-mapping allocated memory address to the head
//! - use a sperated heap track the heap area
#![cfg(target_os = "macos")]

use anyhow::Result;
use pvm::MemoryLike;
use std::collections::BTreeMap;

use crate::TrapInfo;

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
///
/// With this approach, we can avoid host call when access to immediate address.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct Memory {
    /// Read pointer
    read: Vec<u8>,

    /// Write pointer
    write: Vec<u8>,

    /// Stack pointer
    stack: Vec<u8>,

    /// Args pointer
    args: Vec<u8>,

    /// Heap pointer
    heap: Vec<u8>,

    /// The offset between ro-data and stack previous the heap area.
    info: pvm::MemoryInfo,
}

impl Memory {
    /// Create a new memory instance from parser Memory
    pub fn new(pmemory: &pvm::Memory) -> Result<Self> {
        let mut memory = Self {
            info: pmemory.info.clone(),
            ..Default::default()
        };
        memory.init(pmemory)?;
        Ok(memory)
    }

    /// Initialize memory regions from parser memory
    fn init(&mut self, memory: &pvm::Memory) -> Result<()> {
        if !memory.info.read.is_empty() {
            self.read = memory.ro_data()?;
        }

        if !memory.info.write.is_empty() {
            self.write = memory.rw_data()?;
        }

        if !memory.info.stack.is_empty() {
            self.stack = vec![0; memory.info.stack.len()];
        }

        if !memory.info.args.is_empty() {
            self.args = memory.args()?;
        }

        Ok(())
    }

    // Check if a range of heap is allocated
    fn hallocated(&self, start: u32, count: u32) -> bool {
        let end = start + count;
        let size = self.heap.len() as u32;
        if start >= size || end > size {
            return false;
        }

        true
    }

    /// Read bytes from memory with boundary checks
    pub fn read_bytes(&self, addr: u32, len: u32) -> &[u8] {
        let end = addr + len;
        if addr >= self.info.read.start && end <= self.info.read.end {
            let start = (addr - self.info.read.start) as usize;
            return &self.read[start..(start + len as usize)];
        }

        if addr >= self.info.write.start && end <= self.info.write.end {
            let start = (addr - self.info.write.start) as usize;
            return &self.write[start..(start + len as usize)];
        }

        if addr >= self.info.stack.start && end <= self.info.stack.end {
            let start = (addr - self.info.stack.start) as usize;
            return &self.stack[start..(start + len as usize)];
        }

        if addr >= self.info.args.start && end <= self.info.args.end {
            let start = (addr - self.info.args.start) as usize;
            return &self.args[start..(start + len as usize)];
        }

        // heap area
        let Some(addr) = addr.checked_sub(self.info.heap.start) else {
            TrapInfo::fault(addr).raise();
            return &[];
        };

        // we need to handle make the trap here bcz our heap is not mmap.
        if self.hallocated(addr, len) {
            return &self.heap[addr as usize..(addr as usize + len as usize)];
        } else {
            TrapInfo::fault(addr).raise();
            return &[];
        }
    }

    /// Write bytes to memory with boundary checks
    pub fn write_bytes(&mut self, addr: u32, data: &[u8]) {
        let end = addr + data.len() as u32;
        if addr >= self.info.write.start && end <= self.info.write.end {
            let start = (addr - self.info.write.start) as usize;
            self.write[start..(start + data.len())].copy_from_slice(data);
            return;
        }

        if addr >= self.info.stack.start && end <= self.info.stack.end {
            let start = (addr - self.info.stack.start) as usize;
            self.stack[start..(start + data.len())].copy_from_slice(data);
            return;
        }

        // If the address is not in the heap area, throw error
        if addr < self.info.heap.start || end > self.info.heap.end {
            TrapInfo::fault(addr).raise();
            return;
        }

        // heap area
        let Some(haddr) = addr.checked_sub(self.info.heap.start) else {
            TrapInfo::fault(addr).raise();
            return;
        };

        if self.hallocated(haddr, data.len() as u32) {
            self.heap[haddr as usize..(haddr as usize + data.len())].copy_from_slice(data);
        } else {
            TrapInfo::fault(addr).raise();
        }
    }

    /// Convert the virtual memory back to pvm::Memory structure
    pub fn fill(&self, original: &pvm::Memory) -> pvm::Memory {
        let mut memory_map = BTreeMap::new();
        for (&page, (_, perms)) in &original.memory {
            let addr = page * pvm::PAGE_SIZE as u32;
            let mut data = vec![0u8; pvm::PAGE_SIZE as usize];
            let mut size = pvm::PAGE_SIZE as u32;
            if addr < self.info.read.end {
                size = size.min(self.info.read.end - addr);
            } else if addr < self.info.write.end {
                size = size.min(self.info.write.end - addr);
            } else if addr < self.info.heap.start {
                size = size.min(self.info.heap.start - addr);
            } else if addr < self.info.stack.end {
                size = size.min(self.info.stack.end - addr);
            } else if addr < self.info.args.end {
                size = size.min(self.info.args.end - addr);
            }

            // read bytes from memory
            let bytes = self.read_bytes(addr, size);
            data[..bytes.len()].copy_from_slice(bytes);
            if data.iter().any(|&b| b != 0) {
                memory_map.insert(page, (data, *perms));
            }
        }

        pvm::Memory {
            memory: memory_map,
            info: original.info.clone(),
            heap_ptr: original.heap_ptr,
        }
    }
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

impl MemoryLike for Memory {
    fn read(&self, addr: u32, len: u32) -> Result<Vec<u8>> {
        Ok(self.read_bytes(addr, len).to_vec())
    }

    fn write(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        Ok(self.write_bytes(addr, data))
    }

    fn allocate(&mut self, page: u32, count: u32) -> Result<()> {
        if count == 0 {
            return Ok(());
        }

        let start = page * pvm::PAGE_SIZE as u32;
        let size = count * pvm::PAGE_SIZE as u32;
        tracing::debug!("allocating memory: {} - {}", start, size);
        self.heap.resize((start + size) as usize, 0);
        Ok(())
    }

    fn heap_ptr(&self) -> u32 {
        self.info.heap.start + self.heap.len() as u32
    }

    fn set_heap_ptr(&mut self, heap_ptr: u32) {
        self.heap
            .resize((heap_ptr - self.info.heap.start) as usize, 0);
    }
}
