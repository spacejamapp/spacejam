//! Translator context

use crate::Translator;
use cranelift::prelude::*;
use pvm::Program;

/// ExtendedContext memory layout offsets
pub mod offsets {
    /// Size of register array in bytes
    pub const REGISTERS_SIZE: usize = pvm::REGISTER_COUNT * 8;

    /// Offset to gas field (after registers)
    pub const GAS_OFFSET: usize = REGISTERS_SIZE;

    /// Offset to PC field (after registers + gas)
    pub const PC_OFFSET: usize = REGISTERS_SIZE + 8;

    /// Offset to memory pointer (after registers + PC + gas)
    pub const MEMORY_PTR_OFFSET: usize = PC_OFFSET + 8;

    /// Offset to context pointer (after registers + PC + gas + memory pointer)
    pub const CTX_PTR_OFFSET: usize = MEMORY_PTR_OFFSET + pvm::PVM_MEMORY_SIZE as usize;
}

impl Translator<'_> {
    /// Initialize context
    pub fn init_context(&mut self, program: &Program, ctx: Value) {
        self.ctx_ptr = ctx;
        self.builder.declare_var(self.jump, types::I64);
        self.init_registers(&program.registers);
        self.init_memory(ctx, &program.memory);
    }
}
