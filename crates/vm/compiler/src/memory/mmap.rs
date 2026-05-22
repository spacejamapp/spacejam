//! Memory management for PVM programs using mmap for efficient virtual memory
#![cfg(target_os = "linux")]

use anyhow::Result;
use libc::{MAP_ANONYMOUS, MAP_NORESERVE, MAP_PRIVATE, PROT_NONE, PROT_READ, PROT_WRITE};
use pvm::MemoryLike;
use std::{cell::RefCell, collections::BTreeMap, io, ptr};

thread_local! {
    static POOL: RefCell<Option<Region>> = const { RefCell::new(None) };
}

/// memory for PVM programs
#[derive(Debug, Clone)]
pub struct Memory {
    /// Base pointer to the virtual memory region
    base: *mut u8,

    /// Heap pointer
    ///
    /// NOTE: using [u64] for safe mapping in C API
    pub heap_ptr: u32,
}

impl Memory {
    /// Create a new memory instance from parser Memory
    pub fn new(pmemory: &pvm::Memory) -> Result<Self> {
        let region = match POOL.with(|p| p.borrow_mut().take()) {
            Some(r) => r,
            None => Self::mmap_region()?,
        };
        let base = region.0;
        std::mem::forget(region);

        let memory = Memory {
            base,
            heap_ptr: pmemory.heap_ptr,
        };

        memory.init(pmemory)?;
        Ok(memory)
    }

    fn mmap_region() -> Result<Region> {
        let base = unsafe {
            libc::mmap(
                ptr::null_mut(),
                pvm::PVM_MEMORY_SIZE as usize,
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE,
                -1,
                0,
            )
        };
        if base == libc::MAP_FAILED {
            anyhow::bail!(
                "Failed to mmap virtual memory: {}",
                std::io::Error::last_os_error()
            );
        }
        Ok(Region(base as *mut u8))
    }

    /// Initialize memory regions from parser memory
    fn init(&self, memory: &pvm::Memory) -> Result<()> {
        unsafe {
            // Set up read-only data region
            {
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
            {
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

            // Set up stack region as read-write
            {
                let start = memory.info.stack.start as usize;
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
            {
                let args = memory.args()?;
                let start = memory.info.args.start as usize;
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

    /// Read bytes from memory
    #[inline]
    pub fn read_bytes(&self, addr: u32, len: u32) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.base.add(addr as usize), len as usize) }
    }

    /// Write bytes to memory
    #[inline]
    pub fn write_bytes(&mut self, addr: u32, data: &[u8]) {
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.base.add(addr as usize), data.len());
        }
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

impl Drop for Memory {
    fn drop(&mut self) {
        if self.base.is_null() {
            return;
        }

        let size = pvm::PVM_MEMORY_SIZE as usize;
        unsafe {
            libc::madvise(self.base as *mut _, size, libc::MADV_DONTNEED);
            libc::mprotect(self.base as *mut _, size, PROT_NONE);
        }

        let base = self.base;
        self.base = ptr::null_mut();

        POOL.with(|p| {
            let mut slot = p.borrow_mut();
            if slot.is_none() {
                *slot = Some(Region(base));
            } else {
                // Slot already full; drop this region (munmaps via Region::drop).
                drop(Region(base));
            }
        });
    }
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

impl MemoryLike for Memory {
    fn read(&self, addr: u32, len: u32) -> Result<Vec<u8>> {
        Ok(self.read_bytes(addr, len).to_vec())
    }

    fn read_into(&self, addr: u32, buf: &mut [u8]) -> Result<()> {
        pvm::check_range(addr, buf.len() as u32)?;
        let src = self.read_bytes(addr, buf.len() as u32);
        buf.copy_from_slice(src);
        Ok(())
    }

    fn write(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        self.write_bytes(addr, data);
        Ok(())
    }

    fn allocate(&mut self, page: u32, count: u32) -> Result<()> {
        if count == 0 {
            return Ok(());
        }

        for page_num in page..(page + count) {
            let page_addr = (page_num as usize) * (pvm::PAGE_SIZE as usize);
            unsafe {
                if libc::mprotect(
                    self.base.add(page_addr) as *mut _,
                    pvm::PAGE_SIZE as usize,
                    PROT_READ | PROT_WRITE,
                ) != 0
                {
                    anyhow::bail!(
                        "Failed to commit page {} at addr {:#x}: {}",
                        page_num,
                        page_addr,
                        io::Error::last_os_error()
                    );
                }
            }
        }

        Ok(())
    }

    fn heap_ptr(&self) -> u32 {
        self.heap_ptr
    }

    fn set_heap_ptr(&mut self, heap_ptr: u32) {
        self.heap_ptr = heap_ptr;
    }
}

// Per-thread reusable 4 GB guest mapping.
struct Region(*mut u8);

impl Drop for Region {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { libc::munmap(self.0 as *mut _, pvm::PVM_MEMORY_SIZE as usize) };
        }
    }
}
