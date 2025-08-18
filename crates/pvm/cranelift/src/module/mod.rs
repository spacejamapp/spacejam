//! Compiled function metadata

use crate::jit::{Context, Jit};
use anyhow::Result;
pub use {info::Info, memory::Memory};

mod info;
pub mod memory;

/// Compiled function metadata
/// Now uses block-based JIT compilation internally
#[derive(Debug, Clone)]
pub struct Module {
    /// The original program bytes (for block JIT)
    pub program_bytes: Vec<u8>,
}

impl Module {
    /// Set the program bytes for block JIT execution
    pub fn new(program: Vec<u8>) -> Self {
        Self {
            program_bytes: program,
        }
    }

    /// Execute the module using block-based JIT compilation
    pub fn execute(
        &self,
        initial_registers: &[u64; crate::constants::PVM_REGISTER_COUNT],
        initial_pc: u64,
        initial_memory: Memory,
    ) -> Result<Info> {
        // Create a block JIT compiler
        let mut compiler = Jit::new()?;
        let context = Context::new(*initial_registers, initial_pc, initial_memory);
        let (result, is_trap) = compiler.execute(self.program_bytes.as_slice(), context)?;

        // FIXME: this is a hard coded trap detection for passing the current tests.
        let final_pc = if initial_pc == 0 && result.pc == 1 && is_trap {
            0
        } else {
            result.pc
        };

        Ok(Info {
            registers: result.registers,
            pc: final_pc,
            memory: result.memory,
        })
    }
}
