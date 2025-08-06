//! Compiled function metadata

use anyhow::Result;
pub use {context::Context, info::Info, memory::Memory};

mod context;
mod info;
pub mod memory;

/// Compiled function metadata
#[derive(Debug, Clone)]
pub struct Module {
    /// Entry point address of the compiled function
    pub entry_point: *const u8,
    /// Size of the compiled function in bytes
    pub size: usize,
    /// Number of instructions in the original PVM program
    pub instruction_count: usize,
}

impl Module {
    /// Create a new placeholder compiled function
    pub fn new_placeholder() -> Self {
        Self {
            entry_point: std::ptr::null(),
            size: 0,
            instruction_count: 0,
        }
    }

    /// Create a new compiled function with actual data
    pub fn new(entry_point: *const u8, size: usize, instruction_count: usize) -> Self {
        Self {
            entry_point,
            size,
            instruction_count,
        }
    }

    /// Check if the function is a placeholder
    pub fn is_placeholder(&self) -> bool {
        self.entry_point.is_null() && self.size == 0
    }

    /// Execute the compiled module with initial register values, PC, and memory
    pub fn execute(
        &self,
        initial_registers: &[u64; 13],
        initial_pc: u64,
        initial_memory: Memory,
    ) -> Result<Info> {
        if self.is_placeholder() {
            anyhow::bail!("Cannot execute placeholder function");
        }

        unsafe {
            // Create execution context with direct memory reference
            let mut memory_copy = initial_memory.clone();
            let mut context = Context::new(&mut memory_copy);
            context.registers = *initial_registers;
            context.pc = initial_pc;

            let func_ptr = std::mem::transmute::<*const u8, fn(*mut Context)>(self.entry_point);
            func_ptr(&mut context);

            // Apply any recorded memory operations to the memory state
            context
                .apply_memory_operations()
                .map_err(|e| anyhow::anyhow!("Failed to apply memory operations: {}", e))?;

            // Sync linear memory changes back to PVM pages
            context
                .sync_linear_to_pages()
                .map_err(|e| anyhow::anyhow!("Failed to sync linear memory: {}", e))?;

            // Get the final memory state
            let final_memory = memory_copy;

            Ok(Info {
                registers: context.registers,
                pc: context.pc,
                memory: final_memory,
            })
        }
    }
}
