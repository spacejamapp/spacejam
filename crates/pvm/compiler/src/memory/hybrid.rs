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
#[derive(Debug, Clone)]
pub struct Memory {
    /// Head pointer (read-only + read-write)
    head: *mut u8,

    /// Heap pointer
    heap: Vec<u8>,

    /// Trail pointer (stack + args)
    trail: *mut u8,

    /// The offset between ro-data and stack previous the heap area.
    info: pvm::MemoryInfo,
}

impl Memory {
    /// Create a new memory instance from parser Memory
    pub fn new(pmemory: &pvm::Memory) -> Result<Self> {
        let info = &pmemory.info;
        let head = unsafe {
            libc::mmap(
                ptr::null_mut(),
                (info.write.end as usize).max(info.read.end as usize),
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE,
                -1,
                0,
            )
        };

        let trail = unsafe {
            libc::mmap(
                ptr::null_mut(),
                (info.args.end as usize).max(info.stack.end as usize),
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE,
                -1,
                0,
            )
        };

        let memory = Self {
            head: head as *mut u8,
            trail: trail as *mut u8,
            heap: vec![],
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
                if libc::mprotect(self.head.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0
                {
                    anyhow::bail!(
                        "Failed to make region {}-{} writable: {}",
                        memory.info.read.start,
                        memory.info.read.end,
                        io::Error::last_os_error()
                    );
                }

                ptr::copy_nonoverlapping(read.as_ptr(), self.head.add(start), size);
                if libc::mprotect(self.head.add(start) as *mut _, size, PROT_READ) != 0 {
                    anyhow::bail!(
                        "Failed to set read region {}-{} read-only: {}",
                        memory.info.read.start,
                        memory.info.read.end,
                        io::Error::last_os_error()
                    );
                }
            }

            // Set up write region as read-write
            if !memory.info.write.is_empty() {
                let write = memory.rw_data()?;
                let start = memory.info.write.start as usize;
                let size = write.len();
                if libc::mprotect(self.head.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0
                {
                    anyhow::bail!(
                        "Failed to set write region {}-{} writable: {}",
                        memory.info.write.start,
                        memory.info.write.end,
                        io::Error::last_os_error()
                    );
                }

                ptr::copy_nonoverlapping(write.as_ptr(), self.head.add(start), size);
            }

            let offset = memory.info.heap.len();

            // Set up stack region as read-write
            if !memory.info.stack.is_empty() {
                let start = memory.info.stack.start as usize - offset;
                let size = (memory.info.stack.end - memory.info.stack.start) as usize;
                if libc::mprotect(
                    self.trail.add(start) as *mut _,
                    size,
                    PROT_READ | PROT_WRITE,
                ) != 0
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
                if libc::mprotect(
                    self.trail.add(start) as *mut _,
                    size,
                    PROT_READ | PROT_WRITE,
                ) != 0
                {
                    anyhow::bail!(
                        "Failed to make args region writable: {}",
                        io::Error::last_os_error()
                    );
                }

                ptr::copy_nonoverlapping(args.as_ptr(), self.head.add(start), size);
                if libc::mprotect(self.head.add(start) as *mut _, size, PROT_READ) != 0 {
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
    ///
    /// Some condition may confused bcz we need to adapt the tests which
    /// are not standard memory layout, however it's okay since all of
    /// the problems will be catched by the virtual memory.
    pub fn read_bytes(&self, addr: u32, len: u32) -> &[u8] {
        let end = addr + len;
        if (addr >= self.info.read.start && end <= self.info.read.end)
            || (addr >= self.info.write.start && end <= self.info.write.end)
        {
            return unsafe { slice::from_raw_parts(self.head.add(addr as usize), len as usize) };
        }

        if (addr >= self.info.stack.start && end <= self.info.stack.end)
            || (addr >= self.info.args.start && end <= self.info.args.end)
        {
            return unsafe {
                slice::from_raw_parts(
                    self.trail.add(addr as usize - self.info.heap.len()),
                    len as usize,
                )
            };
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
            unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), self.head.add(addr as usize), data.len())
            }

            return;
        }

        if addr >= self.info.stack.start && end <= self.info.stack.end {
            unsafe {
                ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    self.trail.add(addr as usize - self.info.heap.len()),
                    data.len(),
                )
            };

            return;
        }

        // If the address is not in the heap area, throw error
        if !self.info.heap.contains(&addr) || !self.info.heap.contains(&end) {
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
            tracing::error!("heap area not allocated: {} - {}", addr, data.len());
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

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            if !self.head.is_null() {
                libc::munmap(
                    self.head as *mut _,
                    self.info.read.end.max(self.info.write.end) as usize,
                );
            }

            if !self.trail.is_null() {
                libc::munmap(
                    self.trail as *mut _,
                    self.info.args.end.max(self.info.stack.end) as usize,
                );
            }
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
