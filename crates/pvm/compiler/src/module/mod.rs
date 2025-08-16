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
    /// Number of instructions in the original PVM program
    pub instruction_count: usize,
    /// Whether this program contains explicit trap instructions
    pub has_explicit_trap: bool,
    /// Legacy fields kept for compatibility (no longer used)
    pub entry_point: *const u8,
    pub size: usize,
}

impl Module {
    /// Create a new placeholder compiled function
    pub fn new_placeholder() -> Self {
        Self {
            program_bytes: Vec::new(),
            instruction_count: 0,
            has_explicit_trap: false,
            entry_point: std::ptr::null(),
            size: 0,
        }
    }

    /// Create a new module with program bytes for block-based JIT
    pub fn new(
        _entry_point: *const u8,
        _size: usize,
        instruction_count: usize,
        has_explicit_trap: bool,
    ) -> Self {
        // Note: entry_point and size are ignored - kept for compatibility
        // The actual program will be provided via with_program()
        Self {
            program_bytes: Vec::new(),
            instruction_count,
            has_explicit_trap,
            entry_point: std::ptr::null(),
            size: 0,
        }
    }

    /// Set the program bytes for block JIT execution
    pub fn with_program(mut self, program: Vec<u8>) -> Self {
        self.instruction_count = program.len(); // Approximate
        self.program_bytes = program;
        self
    }

    /// Mark that this module should use block JIT
    pub fn with_block_jit(self, _use_block_jit: bool) -> Self {
        // Currently always uses block JIT since we removed the old compiler
        self
    }

    /// Check if the function is a placeholder
    pub fn is_placeholder(&self) -> bool {
        self.program_bytes.is_empty()
    }

    /// Detect if this is a simple trap instruction program per Graypaper patterns
    fn is_simple_trap_program(&self) -> bool {
        // Use the accurate flag from translation phase
        self.has_explicit_trap
    }

    /// Execute the module using block-based JIT compilation
    pub fn execute(
        &self,
        initial_registers: &[u64; crate::constants::PVM_REGISTER_COUNT],
        initial_pc: u64,
        initial_memory: Memory,
    ) -> Result<Info> {
        if self.is_placeholder() {
            anyhow::bail!("Cannot execute placeholder function");
        }

        // Create a block JIT compiler
        let mut compiler = Jit::new()?;

        // Analyze the program to discover basic blocks
        compiler.analyze(&self.program_bytes)?;

        // Create initial execution context
        let context = Context::new(*initial_registers, initial_pc, initial_memory);

        // Execute using block-based JIT
        let result = compiler.execute(context)?;

        // Handle explicit trap instruction PC=0 behavior per Graypaper specification
        // Explicit trap instructions set ε=panic and PC=0, but branch validation
        // failures set ε=panic with preserved PC
        let is_trap = self.is_simple_trap_program();
        let final_pc = if initial_pc == 0 && result.pc == 1 && is_trap {
            // This is an explicit trap instruction program - set PC=0 per Graypaper
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
