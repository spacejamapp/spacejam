//! Memory management for PVM programs using mmap for efficient virtual memory

use anyhow::{bail, Result};
use libc::{
    mmap, mprotect, munmap, MAP_ANONYMOUS, MAP_NORESERVE, MAP_PRIVATE, PROT_NONE, PROT_READ,
    PROT_WRITE,
};
use pvm::MemoryLike;
use std::{collections::BTreeMap, io, ptr};

/// memory for PVM programs
#[derive(Debug, Clone)]
pub struct Memory {
    /// Base pointer to the virtual memory region
    base: *mut u8,
    /// Heap pointer
    pub heap_ptr: u32,
}

impl Memory {
    /// Create a new memory instance from parser Memory
    pub fn new(pmemory: &pvm::Memory) -> Result<Self> {
        let base = unsafe {
            mmap(
                ptr::null_mut(),
                pvm::PVM_MEMORY_SIZE as usize,
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE,
                -1,
                0,
            )
        };

        if base == libc::MAP_FAILED {
            bail!(
                "Failed to mmap virtual memory: {}",
                std::io::Error::last_os_error()
            );
        }

        let memory = Memory {
            base: base as *mut u8,
            heap_ptr: pmemory.heap_ptr,
        };

        memory.init(pmemory)?;
        Ok(memory)
    }

    /// Initialize memory regions from parser memory
    fn init(&self, memory: &pvm::Memory) -> Result<()> {
        unsafe {
            // Set up read-only data region
            {
                let read = memory.ro_data()?;
                let start = memory.info.read.start as usize;
                let size = read.len();
                if mprotect(self.base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0 {
                    bail!(
                        "Failed to make read region writable: {}",
                        io::Error::last_os_error()
                    );
                }

                ptr::copy_nonoverlapping(read.as_ptr(), self.base.add(start), size);
                if mprotect(self.base.add(start) as *mut _, size, PROT_READ) != 0 {
                    bail!(
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
                if mprotect(self.base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0 {
                    bail!(
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
                if mprotect(self.base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0 {
                    bail!(
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
                if mprotect(self.base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0 {
                    bail!(
                        "Failed to make args region writable: {}",
                        io::Error::last_os_error()
                    );
                }

                ptr::copy_nonoverlapping(args.as_ptr(), self.base.add(start), size);
                if mprotect(self.base.add(start) as *mut _, size, PROT_READ) != 0 {
                    bail!(
                        "Failed to set args region read-only: {}",
                        io::Error::last_os_error()
                    );
                }
            }
        }

        Ok(())
    }

    /// base pointer for direct memory access
    #[inline]
    pub fn base(&self) -> *mut u8 {
        self.base
    }

    /// Allocate heap memory
    pub fn allocate(&self, page: u32, count: u32) -> Result<()> {
        if count == 0 {
            return Ok(());
        }

        // Make pages writable
        for page_num in page..(page + count) {
            let page_addr = (page_num as usize) * (pvm::PAGE_SIZE as usize);
            unsafe {
                if mprotect(
                    self.base.add(page_addr) as *mut _,
                    pvm::PAGE_SIZE as usize,
                    PROT_READ | PROT_WRITE,
                ) != 0
                {
                    let err = io::Error::last_os_error();
                    if err.raw_os_error() != Some(libc::EACCES) {
                        bail!("Failed to allocate page {}: {}", page_num, err);
                    }
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

    /// Read a value from memory
    #[inline]
    pub fn read<T: Copy>(&self, addr: u32) -> T {
        unsafe {
            let ptr = self.base.add(addr as usize) as *const T;
            ptr.read_unaligned()
        }
    }

    /// Write a value to memory
    #[inline]
    pub fn write<T>(&mut self, addr: u32, value: T) {
        unsafe {
            let ptr = self.base.add(addr as usize) as *mut T;
            ptr.write_unaligned(value);
        }
    }

    /// Convert the virtual memory back to pvm::Memory structure
    /// This reads the modified memory and creates a new pvm::Memory with the changes
    pub fn fill(&self, original: &pvm::Memory) -> pvm::Memory {
        let mut memory_map = BTreeMap::new();
        for (&page_num, (_, perms)) in &original.memory {
            let page_addr = (page_num as usize) * (pvm::PAGE_SIZE as usize);

            // Read the page data from virtual memory
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

        // Also check for new pages that might have been allocated (heap growth)
        // For now, we'll only sync pages that were already in the original memory

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
            if !self.base.is_null() {
                munmap(self.base as *mut _, pvm::PVM_MEMORY_SIZE as usize);
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
        self.write_bytes(addr, data);
        Ok(())
    }

    fn allocate(&mut self, page: u32, count: u32) -> Result<()> {
        Memory::allocate(self, page, count)?;
        self.heap_ptr = page * (pvm::PAGE_SIZE as u32) + count * (pvm::PAGE_SIZE as u32);
        Ok(())
    }

    fn heap_ptr(&self) -> u32 {
        self.heap_ptr
    }
}
