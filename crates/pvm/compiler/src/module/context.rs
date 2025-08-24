//! Runtime context for block execution
//!
//! TODO: totally refactor the memory part.

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
        let mut mem = vec![0u8; 4096];
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

    /// Extend context to be used in compiled blocks
    pub fn extend(&mut self) -> translator::Context {
        translator::Context {
            registers: self.registers,
            pc: self.pc,
            memory_ptr: self.mem.as_mut_ptr(),
        }
    }
}
