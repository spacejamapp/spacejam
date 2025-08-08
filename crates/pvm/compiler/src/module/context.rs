//! Execution context

use super::memory::Memory;

/// Memory operation for tracking during JIT execution
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryOp {
    pub address: u32,
    pub value: u64,
    pub size: u8, // 1, 2, 4, or 8 bytes
    pub is_write: bool,
}

/// Execution context passed to compiled functions
#[repr(C)]
#[derive(Debug)]
pub struct Context {
    /// PVM registers
    pub registers: [u64; 13],
    /// Program counter
    pub pc: u64,
    /// Linear memory buffer pointer for JIT execution (points to Vec<u8> data)
    pub memory_ptr: *mut u8,
    /// Gas remaining
    pub gas: u64,
    /// Memory operations buffer (for tracking during JIT execution)
    pub memory_ops: [MemoryOp; 64], // Fixed-size buffer for simplicity
    /// Number of recorded memory operations
    pub memory_ops_count: u32,
    /// Whether memory has any allocated pages (for trap detection)
    pub has_memory_pages: u32, // 0 = no pages, 1 = has pages
    /// Linear memory buffer for JIT execution
    pub linear_memory: Vec<u8>,
    /// Reference to the original PVM Memory structure
    pub pvm_memory: *mut Memory,
}

impl Context {
    /// Create a new execution context
    pub fn new(memory: &mut Memory) -> Self {
        let has_pages = if memory.pages.is_empty() { 0 } else { 1 };

        // Create linear memory buffer from PVM pages
        // Allocate up to 1MB for linear memory (should be enough for tests)
        let mut linear_memory = vec![0u8; 0x100000]; // 1MB

        // Copy all pages to linear memory
        for (&page_num, page) in &memory.pages {
            let start_addr = (page_num * 4096) as usize;
            let end_addr = start_addr + 4096;

            // Only copy if within linear memory bounds
            if end_addr <= linear_memory.len() {
                linear_memory[start_addr..end_addr].copy_from_slice(&page.data);
            }
        }

        let mut ctx = Self {
            registers: [0; 13],
            pc: 0,
            memory_ptr: std::ptr::null_mut(), // Will be set below
            gas: 1000000,                     // Default gas limit
            memory_ops: [MemoryOp {
                address: 0,
                value: 0,
                size: 0,
                is_write: false,
            }; 64],
            memory_ops_count: 0,
            has_memory_pages: has_pages,
            linear_memory,
            pvm_memory: memory as *mut Memory,
        };

        // Set memory_ptr to point to the linear memory buffer
        ctx.memory_ptr = ctx.linear_memory.as_mut_ptr();
        ctx
    }

    /// Get PVM memory reference (safe)
    pub fn memory(&self) -> Option<&Memory> {
        if self.pvm_memory.is_null() {
            None
        } else {
            unsafe { Some(&*self.pvm_memory) }
        }
    }

    /// Get mutable PVM memory reference (safe)
    pub fn memory_mut(&mut self) -> Option<&mut Memory> {
        if self.pvm_memory.is_null() {
            None
        } else {
            unsafe { Some(&mut *self.pvm_memory) }
        }
    }

    /// Record a memory operation (called from JIT code)
    pub fn record_memory_op(&mut self, address: u32, value: u64, size: u8, is_write: bool) {
        if self.memory_ops_count < 64 {
            self.memory_ops[self.memory_ops_count as usize] = MemoryOp {
                address,
                value,
                size,
                is_write,
            };
            self.memory_ops_count += 1;
        }
    }

    /// Apply recorded memory operations to the memory
    pub fn apply_memory_operations(&mut self) -> anyhow::Result<()> {
        // Copy the operations and count to avoid borrow checker issues
        let ops_count = self.memory_ops_count as usize;
        let ops = self.memory_ops[0..ops_count].to_vec();

        // Process all operations - collect results for PVM memory writes
        let mut write_results = Vec::new();

        // First pass: write to PVM memory and collect results
        if !self.pvm_memory.is_null() {
            let memory = unsafe { &mut *self.pvm_memory };
            for op in ops.iter() {
                if op.is_write {
                    // Write to PVM memory pages
                    let result = match op.size {
                        1 => memory.write_u8(op.address, op.value as u8),
                        2 => memory.write_u16(op.address, op.value as u16),
                        4 => memory.write_u32(op.address, op.value as u32),
                        8 => memory.write_u64(op.address, op.value),
                        _ => Ok(()), // Invalid size, ignore
                    };

                    // If memory write fails, this indicates a memory access trap condition
                    if result.is_err() {
                        // Memory access traps should set PC=0 per Graypaper specification
                        // This is different from branch validation failures which preserve PC
                        self.pc = 0;
                        // Don't return error - let execution continue with PC=0
                        return Ok(());
                    }
                    write_results.push(result);
                } else {
                    write_results.push(Ok(())); // Read operations always succeed for tracking
                }
            }
        }

        // Second pass: update linear memory buffer for successful writes
        for (i, op) in ops.iter().enumerate() {
            if op.is_write && i < write_results.len() && write_results[i].is_ok() {
                let addr = op.address as usize;
                match op.size {
                    1 => {
                        if addr < self.linear_memory.len() {
                            self.linear_memory[addr] = op.value as u8;
                        }
                    }
                    2 => {
                        if addr + 1 < self.linear_memory.len() {
                            let bytes = (op.value as u16).to_le_bytes();
                            self.linear_memory[addr..addr + 2].copy_from_slice(&bytes);
                        }
                    }
                    4 => {
                        if addr + 3 < self.linear_memory.len() {
                            let bytes = (op.value as u32).to_le_bytes();
                            self.linear_memory[addr..addr + 4].copy_from_slice(&bytes);
                        }
                    }
                    8 => {
                        if addr + 7 < self.linear_memory.len() {
                            let bytes = op.value.to_le_bytes();
                            self.linear_memory[addr..addr + 8].copy_from_slice(&bytes);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Sync linear memory changes back to PVM pages
    pub fn sync_linear_to_pages(&mut self) -> anyhow::Result<()> {
        if !self.pvm_memory.is_null() {
            let memory = unsafe { &mut *self.pvm_memory };
            // Copy modified linear memory back to pages
            for (&page_num, page) in memory.pages.iter_mut() {
                let start_addr = (page_num * 4096) as usize;
                let end_addr = start_addr + 4096;

                // Only copy if within linear memory bounds
                if end_addr <= self.linear_memory.len() {
                    page.data
                        .copy_from_slice(&self.linear_memory[start_addr..end_addr]);
                }
            }
        }
        Ok(())
    }
}
