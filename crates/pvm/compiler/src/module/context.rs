//! Runtime context for block execution

use crate::{module::Memory, ExecResult};
use anyhow::Result;
use translator::{
    access, BITS_PER_WORD, EXTRA_PAGES_MARGIN, LINEAR_MEMORY_SIZE, PAGE_SIZE, PVM_REGISTER_COUNT,
};

/// Runtime context for block execution
#[derive(Debug, Clone)]
pub struct Context {
    pub registers: [u64; PVM_REGISTER_COUNT],
    pub pc: u64,
    pub memory: Memory,
    pub linear_mem: Vec<u8>,
}

impl Context {
    /// Create new context
    pub fn new(regs: [u64; PVM_REGISTER_COUNT], pc: u64, mem: Memory) -> Self {
        let mut linear_mem = vec![0u8; LINEAR_MEMORY_SIZE];
        for (&page_num, page) in &mem.pages {
            let start = (page_num as usize) * (PAGE_SIZE as usize);
            let end = start + page.data.len();
            if end <= linear_mem.len() {
                linear_mem[start..end].copy_from_slice(&page.data);
            }
        }

        Self {
            registers: regs,
            pc,
            memory: mem,
            linear_mem,
        }
    }

    /// Generate page allocation bitmap for runtime boundary checking
    pub fn generate_page_bitmap(&self) -> (Vec<u64>, Vec<u8>) {
        let max_page = self.memory.pages.keys().max().copied().unwrap_or(0);
        let bitmap_size = ((max_page + BITS_PER_WORD as u32) / BITS_PER_WORD as u32) as usize;
        let mut bitmap = vec![0u64; bitmap_size];

        // Ensure access array is large enough to handle boundary checking beyond max_page
        // We need to account for multi-byte stores that may access pages beyond max_page
        let access_size = (max_page + EXTRA_PAGES_MARGIN + 1) as usize;
        let mut access = vec![access::INACCESSIBLE; access_size]; // Default: inaccessible

        tracing::debug!(
            "Page bitmap generation: max_page={}, bitmap_size={}, access_size={}",
            max_page,
            bitmap_size,
            access_size
        );
        tracing::debug!(
            "Allocated pages: {:?}",
            self.memory.pages.keys().collect::<Vec<_>>()
        );

        for (&page_num, page) in &self.memory.pages {
            let word_idx = page_num / BITS_PER_WORD as u32;
            let bit_idx = page_num % BITS_PER_WORD as u32;
            tracing::debug!(
                "Page {}: word_idx={}, bit_idx={}, access={}",
                page_num,
                word_idx,
                bit_idx,
                page.access
            );
            if (word_idx as usize) < bitmap.len() {
                bitmap[word_idx as usize] |= 1u64 << bit_idx;
                if (page_num as usize) < access.len() {
                    access[page_num as usize] = page.access;
                }
            }
        }

        (bitmap, access)
    }

    /// Sync linear memory back to pages
    ///
    /// NOTE: This method only detects page faults for writes that actually occurred.
    /// It cannot detect cases where a store instruction should have written more bytes
    /// but was truncated due to page boundaries. For proper page fault detection,
    /// boundary checking should be implemented in the store visitor functions.
    pub fn sync(&mut self) -> Result<()> {
        let page_size = PAGE_SIZE as usize;

        // Check for any writes to unallocated pages
        for page_addr in (0..self.linear_mem.len()).step_by(page_size) {
            let page_num = (page_addr / page_size) as u32;
            let page_end = (page_addr + page_size).min(self.linear_mem.len());

            if !self.memory.pages.contains_key(&page_num) {
                let page_data = &self.linear_mem[page_addr..page_end];
                if page_data.iter().any(|&b| b != 0) {
                    anyhow::bail!("Page fault: write to unallocated page {}", page_num);
                }
            }
        }

        // Check for read-only violations and copy back changes
        for (&page_num, page) in &mut self.memory.pages {
            let start = (page_num as usize) * page_size;
            let end = start + page.data.len();

            if end <= self.linear_mem.len() {
                let orig = &page.data[..];
                let new = &self.linear_mem[start..end];

                if orig != new {
                    // Check for read-only page violations
                    if page.access != access::MUTABLE {
                        anyhow::bail!("Page fault: write to read-only page {}", page_num);
                    }

                    // Copy changes back
                    page.data.copy_from_slice(new);
                }
            }
        }

        Ok(())
    }
}

/// Extended context for compiled blocks
#[repr(C)]
pub struct ExtendedContext {
    pub registers: [u64; PVM_REGISTER_COUNT],
    pub pc: u64,
    pub memory_ptr: *mut u8,
    pub page_bitmap: *const u64, // Bitmap of allocated pages
    pub page_access: *const u8,  // Access permissions per page
    pub result: ExecResult,
    pub pc_managed: bool, // Flag indicating instruction handled PC directly
}
