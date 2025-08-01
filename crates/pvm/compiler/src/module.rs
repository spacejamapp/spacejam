//! Compiled function metadata

use anyhow::Result;

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

    /// Execute the compiled module with initial register values
    pub fn execute(&self, initial_registers: &[u64; 13]) -> Result<[u64; 13]> {
        if self.is_placeholder() {
            anyhow::bail!("Cannot execute placeholder function");
        }

        unsafe {
            // Copy initial registers to mutable array
            let mut registers = [0u64; 13];
            registers.copy_from_slice(initial_registers);

            let func_ptr = std::mem::transmute::<*const u8, fn(*mut u64)>(self.entry_point);
            func_ptr(registers.as_mut_ptr());
            Ok(registers)
        }
    }
}
