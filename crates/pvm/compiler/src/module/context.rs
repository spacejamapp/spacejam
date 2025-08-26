//! Runtime context for block execution
//!
//! TODO: totally refactor the memory part.

use crate::Memory;
use anyhow::Result;

/// Runtime context for block execution
pub struct Context {
    pub registers: [u64; pvm::REGISTER_COUNT],
    pub pc: u64,
    pub gas: u64,
    pub memory: pvm::Memory,
    pub memory_impl: Option<Memory>,
}

impl Context {
    /// Create new context
    pub fn new(regs: [u64; pvm::REGISTER_COUNT], pc: u64, memory: pvm::Memory) -> Self {
        Self {
            registers: regs,
            pc,
            gas: 0,
            memory,
            memory_impl: None,
        }
    }

    /// Extend context to be used in compiled blocks
    pub fn extend(&mut self) -> Result<translator::Context> {
        let memory_impl = Memory::new(&self.memory)?;
        let memory_ptr = memory_impl.base();
        self.memory_impl = Some(memory_impl);

        Ok(translator::Context {
            registers: self.registers,
            pc: self.pc,
            gas: self.gas,
            memory_ptr: memory_ptr as _,
        })
    }

    /// Sync memory changes back from virtual memory to pvm::Memory
    pub fn sync(&mut self) {
        if let Some(ref memory_impl) = self.memory_impl {
            self.memory = memory_impl.memory(&self.memory);
        }
    }
}
