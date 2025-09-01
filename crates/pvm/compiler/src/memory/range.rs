//! Memory management for PVM programs on macOS
//!
//! ## macOS
//!
//! since macOS doesn't support large virtual memory, thus we use
//! a range table to implement the memory management on macOS.
//!
//! - re-mapping allocated memory address to the head
//! - use a separated heap track the heap area
#![cfg(target_os = "macos")]

use crate::TrapInfo;
use anyhow::Result;
use pvm::MemoryLike;
use std::collections::BTreeMap;

/// Hybrid memory management for PVM programs on macOS
///
/// the original PVM memory layout is as follows:
///
/// [ [ro data] [rw data] [heap] [stack] [args] ]
///
/// while in our hybrid approach, we re-map the allocated memory address to the head
/// and use a separated heap to track the heap area.
///
/// [ [rw data] [ro data] [stack] [args] ] [ heap ]
///
/// With this approach, we can avoid host call when access to immediate address.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct Memory {
    /// Base pointer to the memory
    base: Box<[u8]>,

    /// Heap pointer
    heap: Vec<u8>,

    /// The offset between ro-data and stack previous the heap area.
    info: pvm::MemoryInfo,
}

impl Memory {
    /// Create a new memory instance from parser Memory
    pub fn new(pmemory: &pvm::Memory) -> Result<Self> {
        tracing::debug!("memory info: {:?}", pmemory.info);
        let mut data = vec![];
        data.extend_from_slice(&pmemory.ro_data()?);
        data.extend_from_slice(&pmemory.rw_data()?);
        data.extend_from_slice(&vec![0; pmemory.info.stack.len()]);
        data.extend_from_slice(&pmemory.args()?);

        Ok(Self {
            base: data.into_boxed_slice(),
            info: pmemory.info.clone(),
            heap: vec![],
        })
    }

    /// Read bytes from memory with boundary checks
    pub fn read_bytes(&self, addr: u32, len: u32) -> Vec<u8> {
        if len == 0 {
            return vec![];
        }

        let end = addr + len;
        let mut ptr = 0;
        if addr >= self.info.read.start && end <= self.info.read.end {
            let start = (addr - self.info.read.start) as usize;
            return self.base[start..(start + len as usize)].to_vec();
        }

        ptr += self.info.read.len();
        if addr >= self.info.write.start && end <= self.info.write.end {
            let start = (addr - self.info.write.start) as usize + ptr;
            return self.base[start..(start + len as usize)].to_vec();
        }

        ptr += self.info.write.len();
        if addr >= self.info.stack.start && end <= self.info.stack.end {
            let start = (addr - self.info.stack.start) as usize + ptr;
            return self.base[start..(start + len as usize)].to_vec();
        }

        ptr += self.info.stack.len();
        if addr >= self.info.args.start && end <= self.info.args.end {
            let start = (addr - self.info.args.start) as usize + ptr;
            return self.base[start..(start + len as usize)].to_vec();
        }

        // reading data from heap or from mutiple regions
        //
        // NOTE: for mutiple regions, we only support (write + heap) atm.
        let mut bytes = vec![];
        let mut addr = addr;
        if addr < self.info.heap.start {
            let start = (addr - self.info.write.start + self.info.read.len() as u32) as usize;
            let end = self.info.read.len() + self.info.write.len();
            bytes.extend_from_slice(&self.base[start..end]);
            addr = self.info.write.end;
        }

        if end > self.info.heap.start + self.heap.len() as u32 {
            TrapInfo::fault(addr).raise();
            return vec![];
        }

        let len = len - bytes.len() as u32;
        addr = addr - self.info.heap.start;
        bytes.extend_from_slice(&self.heap[addr as usize..(addr as usize + len as usize)]);
        bytes
    }

    /// Write bytes to memory with boundary checks
    pub fn write_bytes(&mut self, addr: u32, data: &[u8]) {
        if data.len() == 0 {
            return;
        }

        let end = addr + data.len() as u32;
        let mut ptr = self.info.read.len();
        if addr >= self.info.write.start && end <= self.info.write.end {
            let start = (addr - self.info.write.start) as usize + ptr;
            self.base[start..(start + data.len())].copy_from_slice(data);
            return;
        }

        ptr += self.info.write.len();
        if addr >= self.info.stack.start && end <= self.info.stack.end {
            let start = (addr - self.info.stack.start) as usize + ptr;
            self.base[start..(start + data.len())].copy_from_slice(data);
            return;
        }

        // NOTE: for mutiple regions, we only support (write + heap) atm.
        let mut addr = addr;
        let mut written = 0;
        if addr < self.info.heap.start {
            let wstart = (addr - self.info.write.start + self.info.read.len() as u32) as usize;
            let wend = self.info.read.len() + self.info.write.len();
            let size = (wend - wstart) as usize;
            self.base[wstart..wend].copy_from_slice(&data[..size]);
            written += size;
            addr = self.info.write.end;
        }

        // If the address is not in the heap area, throw error
        if end > self.info.heap.start + self.heap.len() as u32 {
            TrapInfo::fault(addr).raise();
            return;
        }

        addr = addr - self.info.heap.start;
        let len = data.len() - written;
        self.heap[addr as usize..(addr as usize + len)].copy_from_slice(&data[written..]);
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
            data[..bytes.len()].copy_from_slice(&bytes);
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
        self.write_bytes(addr, data);
        Ok(())
    }

    fn allocate(&mut self, page: u32, count: u32) -> Result<()> {
        if count == 0 {
            return Ok(());
        }

        let address = page * pvm::PAGE_SIZE as u32;
        let Some(start) = address.checked_sub(self.info.heap.start) else {
            TrapInfo::fault(page).raise();
            return Ok(());
        };

        let size = count * pvm::PAGE_SIZE as u32;
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
