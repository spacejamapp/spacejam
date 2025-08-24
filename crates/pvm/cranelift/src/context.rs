//! Translator context

use crate::Translator;
use cranelift::prelude::*;
use pvm::Program;

/// ExtendedContext memory layout offsets
pub mod offsets {
    /// Size of register array in bytes
    pub const REGISTERS_SIZE: usize = pvm::REGISTER_COUNT * 8;

    /// Offset to PC field (after registers)
    pub const PC_OFFSET: usize = REGISTERS_SIZE;

    /// Offset to memory pointer (after registers + PC)
    pub const MEMORY_PTR_OFFSET: usize = REGISTERS_SIZE + 8;
}

/// The context of the translator.
pub struct Context {
    /// The registers of the context.
    pub registers: [u64; pvm::REGISTER_COUNT],

    /// TODO: remove pc in production
    pub pc: u64,

    /// The memory pointer
    pub memory_ptr: *mut u8,
}

impl Translator<'_> {
    /// Initialize context
    pub fn init_context(&mut self, program: &Program, ctx: Value) {
        self.ctx_ptr = ctx;
        self.init_registers(&program.registers);
        self.init_memory(ctx, &program.memory);
    }
}
