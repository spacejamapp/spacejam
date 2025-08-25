//! Memory management for PVM programs using mmap for efficient virtual memory
//!
//! This module provides a 4GB virtual address space using mmap with lazy allocation.
//! Physical memory is only allocated when pages are actually accessed, typically
//! using <100KB while providing the full 4GB address space required by PVM.

use anyhow::{bail, Result};
use libc::{
    mmap, mprotect, munmap, MAP_ANONYMOUS, MAP_NORESERVE, MAP_PRIVATE, PROT_NONE, PROT_READ,
    PROT_WRITE,
};
use std::ptr;

/// Memory for PVM programs - just a base pointer to 4GB virtual space
pub struct Memory {
    /// Base pointer to mmap'd 4GB virtual memory region
    base: *mut u8,
}

impl Memory {
    /// Create a new memory instance from parser Memory
    pub fn new(parser_memory: &pvm::Memory) -> Result<Self> {
        // Allocate 4GB virtual address space (no physical memory yet!)
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
                "Failed to mmap 4GB virtual memory: {}",
                std::io::Error::last_os_error()
            );
        }

        let memory = Memory {
            base: base as *mut u8,
        };

        memory.init_from_parser(parser_memory)?;
        Ok(memory)
    }

    /// Initialize memory regions from parser memory
    fn init_from_parser(&self, parser_memory: &pvm::Memory) -> Result<()> {
        // Iterate through all pages in parser memory and set up protection + copy data
        for (&page_num, (page_data, writable)) in &parser_memory.memory {
            let page_addr = (page_num as usize) * (pvm::PAGE_SIZE as usize);
            let page_size = pvm::PAGE_SIZE as usize;

            // Set protection based on writability
            let prot = if *writable {
                PROT_READ | PROT_WRITE
            } else {
                PROT_READ
            };

            unsafe {
                if mprotect(self.base.add(page_addr) as *mut _, page_size, prot) != 0 {
                    bail!(
                        "Failed to mprotect page {}: {}",
                        page_num,
                        std::io::Error::last_os_error()
                    );
                }

                // Copy page data
                if !page_data.is_empty() {
                    let copy_len = page_data.len().min(page_size);
                    ptr::copy_nonoverlapping(
                        page_data.as_ptr(),
                        self.base.add(page_addr),
                        copy_len,
                    );
                }
            }
        }

        Ok(())
    }

    /// Get the base pointer for direct memory access
    #[inline]
    pub fn base(&self) -> *mut u8 {
        self.base
    }

    /// Allocate heap memory
    pub fn sbrk(&self, page: u32, count: u32) -> Result<()> {
        if count == 0 {
            return Ok(());
        }

        // Make pages accessible
        for page_num in page..(page + count) {
            let page_addr = (page_num as usize) * (pvm::PAGE_SIZE as usize);

            unsafe {
                if mprotect(
                    self.base.add(page_addr) as *mut _,
                    pvm::PAGE_SIZE as usize,
                    PROT_READ | PROT_WRITE,
                ) != 0
                {
                    let err = std::io::Error::last_os_error();
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
    pub unsafe fn read_bytes(&self, addr: u32, len: u32) -> &[u8] {
        std::slice::from_raw_parts(self.base.add(addr as usize), len as usize)
    }

    /// Write bytes to memory
    #[inline]
    pub unsafe fn write_bytes(&mut self, addr: u32, data: &[u8]) {
        ptr::copy_nonoverlapping(data.as_ptr(), self.base.add(addr as usize), data.len());
    }

    /// Read a value from memory
    #[inline]
    pub unsafe fn read<T: Copy>(&self, addr: u32) -> T {
        let ptr = self.base.add(addr as usize) as *const T;
        ptr.read_unaligned()
    }

    /// Write a value to memory
    #[inline]
    pub unsafe fn write<T>(&mut self, addr: u32, value: T) {
        let ptr = self.base.add(addr as usize) as *mut T;
        ptr.write_unaligned(value);
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            // Unmap the entire 4GB virtual memory region
            if self.base != ptr::null_mut() {
                munmap(self.base as *mut _, pvm::PVM_MEMORY_SIZE as usize);
            }
        }
    }
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}
