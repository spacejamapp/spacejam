//! Memory management for PVM programs on macOS
//!
//! ## macOS
//!
//! since macOS doesn't support virtual memory larger than 2.5GB thus we use
//! a hybrid approach to implement the memory management on macOS.
//!
//! - re-mapping allocated memory address to the head
//! - use a sperated heap track the heap area

use anyhow::Result;
use libc::{MAP_ANONYMOUS, MAP_NORESERVE, MAP_PRIVATE, PROT_NONE, PROT_READ, PROT_WRITE};
use pvm::MemoryLike;
use std::{collections::BTreeMap, io, ptr, slice};

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
    info: pvm::MemoryInfo,
}

impl Memory {
    /// Create a new memory instance from parser Memory
    pub fn new(pmemory: &pvm::Memory) -> Result<Self> {
        let info = &pmemory.info;
        tracing::debug!("memory info: {:?}", info);
        let size = info.args.len()
            + info.read.len()
            + info.write.len()
            + info.stack.len()
            + info.args.len();
        let base = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE,
                -1,
                0,
            )
        };

        let memory = Self {
            base: base as *mut u8,
            heap: vec![0; info.heap.end as usize - info.heap.start as usize],
            info: info.clone(),
        };

        memory.init(pmemory)?;
        Ok(memory)
    }

    /// Initialize memory regions from parser memory
    fn init(&self, memory: &pvm::Memory) -> Result<()> {
        unsafe {
            // Set up read-only data region
            if !memory.info.read.is_empty() {
                let read = memory.ro_data()?;
                let start = memory.info.read.start as usize;
                let size = read.len();
                if libc::mprotect(self.base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0
                {
                    anyhow::bail!(
                        "Failed to make read region writable: {}",
                        io::Error::last_os_error()
                    );
                }

                ptr::copy_nonoverlapping(read.as_ptr(), self.base.add(start), size);
                if libc::mprotect(self.base.add(start) as *mut _, size, PROT_READ) != 0 {
                    anyhow::bail!(
                        "Failed to set read region read-only: {}",
                        io::Error::last_os_error()
                    );
                }
            }

            // Set up write region as read-write
            if !memory.info.write.is_empty() {
                let write = memory.rw_data()?;
                let start = memory.info.write.start as usize;
                let size = write.len();
                if libc::mprotect(self.base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0
                {
                    anyhow::bail!(
                        "Failed to set write region protection: {}",
                        io::Error::last_os_error()
                    );
                }

                ptr::copy_nonoverlapping(write.as_ptr(), self.base.add(start), size);
            }

            let offset = memory.info.heap.len();

            // Set up stack region as read-write
            if !memory.info.stack.is_empty() {
                let start = memory.info.stack.start as usize - offset;
                let size = (memory.info.stack.end - memory.info.stack.start) as usize;
                if libc::mprotect(self.base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0
                {
                    anyhow::bail!(
                        "Failed to set stack region protection: {}",
                        io::Error::last_os_error()
                    );
                }
            }

            // Set up args region as read-only
            if !memory.info.args.is_empty() {
                let args = memory.args()?;
                let start = memory.info.args.start as usize - offset;
                let size = args.len();
                if libc::mprotect(self.base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0
                {
                    anyhow::bail!(
                        "Failed to make args region writable: {}",
                        io::Error::last_os_error()
                    );
                }

                ptr::copy_nonoverlapping(args.as_ptr(), self.base.add(start), size);
                if libc::mprotect(self.base.add(start) as *mut _, size, PROT_READ) != 0 {
                    anyhow::bail!(
                        "Failed to set args region read-only: {}",
                        io::Error::last_os_error()
                    );
                }
            }
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
    pub fn read_bytes(&self, addr: u32, len: u32) -> Result<&[u8]> {
        tracing::info!("reading bytes from memory: {} - {}", addr, len);
        let end = addr + len;
        if addr < self.info.read.start || addr > self.info.args.end || end > self.info.args.end {
            anyhow::bail!("Invalid address: {}", addr);
        }

        if addr < self.info.heap.start && end < self.info.heap.start {
            return Ok(unsafe {
                slice::from_raw_parts(self.base.add(addr as usize), len as usize)
            });
        }

        if addr > self.info.heap.end {
            return Ok(unsafe {
                slice::from_raw_parts(
                    self.base.add(addr as usize - self.info.heap.len()),
                    len as usize,
                )
            });
        }

        // heap area
        let addr = addr - self.info.heap.start;
        if self.hallocated(addr, len) {
            Ok(&self.heap[addr as usize..(addr as usize + len as usize)])
        } else {
            anyhow::bail!("address not allocated: {}", addr);
        }
    }

    /// Write bytes to memory with boundary checks
    pub fn write_bytes(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        let end = addr + data.len() as u32;
        if addr < self.info.read.start || addr > self.info.args.end || end > self.info.args.end {
            anyhow::bail!("address not accessible: {}", addr);
        }

        if addr > self.info.write.start && end < self.info.write.end {
            return Ok(unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), self.base.add(addr as usize), data.len())
            });
        }

        if addr > self.info.stack.start && end < self.info.stack.end {
            return Ok(unsafe {
                ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    self.base.add(addr as usize - self.info.heap.len()),
                    data.len(),
                )
            });
        }

        // If the address is not in the heap area, throw error
        if !self.info.heap.contains(&addr) || !self.info.heap.contains(&end) {
            anyhow::bail!("address not accessible: {}", addr);
        }

        // heap area
        let addr = addr - self.info.heap.start;
        if self.hallocated(addr, data.len() as u32) {
            unsafe {
                ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    self.heap.as_mut_ptr().add(addr as usize),
                    data.len(),
                );
            }
        } else {
            anyhow::bail!("address not allocated: {}", addr);
        }

        Ok(())
    }

    /// Convert the virtual memory back to pvm::Memory structure
    pub fn fill(&self, original: &pvm::Memory) -> pvm::Memory {
        let mut memory_map = BTreeMap::new();
        for (&page_num, (_, perms)) in &original.memory {
            let page_addr = (page_num as usize) * (pvm::PAGE_SIZE as usize);
            let mut page_data = vec![0u8; pvm::PAGE_SIZE as usize];
            unsafe {
                ptr::copy_nonoverlapping(
                    self.base.add(page_addr),
                    page_data.as_mut_ptr(),
                    pvm::PAGE_SIZE as usize,
                );
            }

            // Only store non-zero pages
            if page_data.iter().any(|&b| b != 0) {
                memory_map.insert(page_num, (page_data, *perms));
            }
        }

        pvm::Memory {
            memory: memory_map,
            info: original.info.clone(),
            heap_ptr: original.heap_ptr,
        }
    }
}

impl MemoryLike for Memory {
    fn read(&self, addr: u32, len: u32) -> Result<Vec<u8>> {
        Ok(self.read_bytes(addr, len)?.to_vec())
    }

    fn write(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        self.write_bytes(addr, data)
    }

    fn allocate(&mut self, page: u32, count: u32) -> Result<()> {
        if count == 0 {
            return Ok(());
        }

        let start = page * pvm::PAGE_SIZE as u32 - self.info.heap.start;
        let size = count * pvm::PAGE_SIZE as u32;
        self.write_bytes(start, &vec![0; size as usize])?;
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
