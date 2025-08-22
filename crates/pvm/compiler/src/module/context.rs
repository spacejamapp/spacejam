//! Runtime context for block execution

use crate::ExecResult;
use anyhow::Result;
use translator::{access, BITS_PER_WORD};

/// Linear memory size for JIT execution (1MB)
pub const LINEAR_MEMORY_SIZE: usize = 0x100000;

/// Extra pages to allocate in access array for boundary checking safety
pub const EXTRA_PAGES_MARGIN: u32 = 64;

/// Runtime context for block execution
#[derive(Debug, Clone)]
pub struct Context {
    pub registers: [u64; pvm::REGISTER_COUNT],
    pub pc: u64,
    pub memory: pvm::Memory,
    pub mem: Vec<u8>,
}

impl Context {
    /// Create new context
    pub fn new(regs: [u64; pvm::REGISTER_COUNT], pc: u64, memory: pvm::Memory) -> Self {
        let mut mem = vec![0u8; LINEAR_MEMORY_SIZE];
        for (&page_num, (page_data, _)) in &memory.memory {
            let start = (page_num as usize) * (pvm::PAGE_SIZE as usize);
            let end = start + page_data.len();
            if end <= mem.len() {
                mem[start..end].copy_from_slice(page_data);
            }
        }

        Self {
            registers: regs,
            pc,
            memory,
            mem,
        }
    }

    /// Generate page allocation bitmap for runtime boundary checking
    pub fn generate_page_bitmap(&self) -> (Vec<u64>, Vec<u8>) {
        let max_page = self.memory.memory.keys().max().copied().unwrap_or(0);
        let bitmap_size = ((max_page + BITS_PER_WORD as u32) / BITS_PER_WORD as u32) as usize;
        let mut bitmap = vec![0u64; bitmap_size];

        // generate access array
        let access_size = (max_page + EXTRA_PAGES_MARGIN + 1) as usize;
        let mut access = vec![access::INACCESSIBLE; access_size];
        for (&page_num, (_, writable)) in &self.memory.memory {
            let word_idx = page_num / BITS_PER_WORD as u32;
            let bit_idx = page_num % BITS_PER_WORD as u32;
            if (word_idx as usize) < bitmap.len() {
                bitmap[word_idx as usize] |= 1u64 << bit_idx;
                if (page_num as usize) < access.len() {
                    access[page_num as usize] = if *writable {
                        access::MUTABLE
                    } else {
                        access::IMMUTABLE
                    };
                }
            }
        }

        (bitmap, access)
    }

    /// Sync linear memory back to pages
    pub fn sync(&mut self) -> Result<()> {
        let page_size = pvm::PAGE_SIZE as usize;

        // Check for any writes to unallocated pages
        for page_addr in (0..self.mem.len()).step_by(page_size) {
            let page_num = (page_addr / page_size) as u32;
            let page_end = (page_addr + page_size).min(self.mem.len());
            if !self.memory.memory.contains_key(&page_num) {
                let page_data = &self.mem[page_addr..page_end];
                if page_data.iter().any(|&b| b != 0) {
                    anyhow::bail!("Page fault: write to unallocated page {}", page_num);
                }
            }
        }

        // Check for read-only violations and copy back changes
        for (&page_num, (page_data, writable)) in &mut self.memory.memory {
            let start = (page_num as usize) * page_size;
            let end = start + page_data.len();

            if end <= self.mem.len() {
                let orig = &page_data[..];
                let new = &self.mem[start..end];

                if orig != new {
                    // Check for read-only page violations
                    if !*writable {
                        anyhow::bail!("Page fault: write to read-only page {}", page_num);
                    }

                    // Copy changes back
                    page_data.copy_from_slice(new);
                }
            }
        }

        Ok(())
    }

    /// Extend context to be used in compiled blocks
    pub fn extend(&mut self) -> ExtendedContext {
        let (page_bitmap, page_access) = self.generate_page_bitmap();
        ExtendedContext {
            registers: self.registers,
            pc: self.pc,
            memory_ptr: self.mem.as_mut_ptr(),
            page_bitmap: page_bitmap.as_ptr(),
            page_access: page_access.as_ptr(),
            result: ExecResult::Continue,
            pc_managed: false,
        }
    }
}

/// Extended context for compiled blocks
#[repr(C)]
pub struct ExtendedContext {
    pub registers: [u64; pvm::REGISTER_COUNT],
    pub pc: u64,
    pub memory_ptr: *mut u8,
    pub page_bitmap: *const u64,
    pub page_access: *const u8,
    pub result: ExecResult,
    pub pc_managed: bool,
}
