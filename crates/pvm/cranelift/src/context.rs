//! Translator context

use crate::Translator;
use cranelift::prelude::*;
use pvm::Program;

/// ExtendedContext memory layout offsets
pub mod offsets {
    /// Size of register array in bytes
    pub const REGISTERS_SIZE: i32 = (pvm::REGISTER_COUNT as i32) * 8;

    /// Offset to gas field (after registers)
    pub const GAS_OFFSET: i32 = REGISTERS_SIZE;

    /// Offset to PC field (after registers + gas)
    pub const PC_OFFSET: i32 = REGISTERS_SIZE + 8;

    /// Offset to memory pointer (after registers + PC + gas)
    pub const HEAP_PTR_OFFSET: i32 = PC_OFFSET + 8;

    /// Offset to memory pointer (after registers + PC + gas)
    pub const MEMORY_PTR_OFFSET: i32 = HEAP_PTR_OFFSET + 8;
}

/// Constants pool with Single Static Assignment Values
pub struct Pool {
    /// The context pointer
    pub ctx: Value,

    /// The memory pointer
    pub memory: Value,

    /// The heap pointer
    pub heapp: Value,
}

impl Translator<'_> {
    /// Initialize context
    pub fn init_context(&mut self, program: &Program, ctx: Value) {
        tracing::debug!("memory info: {:?}", program.memory.info);
        self.pool = Pool {
            memory: self.builder.ins().load(
                types::I64,
                MemFlags::trusted(),
                ctx,
                offsets::MEMORY_PTR_OFFSET,
            ),
            heapp: self.builder.ins().load(
                types::I64,
                MemFlags::trusted(),
                ctx,
                offsets::HEAP_PTR_OFFSET,
            ),
            ctx,
        };
    }
}
