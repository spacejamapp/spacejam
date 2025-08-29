//! Translator context

use crate::Translator;
use core::ops::Range;
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
    pub heap: Value,

    /// The heap range
    pub hrange: Range<Value>,
}

impl Translator<'_> {
    /// Initialize context
    pub fn init_context(&mut self, program: &Program, ctx: Value) {
        self.builder.declare_var(self.jump, types::I64);
        self.pool = Pool {
            memory: self.builder.ins().load(
                types::I64,
                MemFlags::trusted(),
                ctx,
                offsets::MEMORY_PTR_OFFSET,
            ),
            heap: self.builder.ins().load(
                types::I64,
                MemFlags::trusted(),
                ctx,
                offsets::HEAP_PTR_OFFSET,
            ),
            hrange: self
                .builder
                .ins()
                .iconst(types::I64, program.memory.info.heap.start as i64)
                ..self.builder.ins().iconst(
                    types::I64,
                    (program.memory.info.heap.start + program.memory.info.heap.end) as i64,
                ),
            ctx,
        };

        self.init_registers(&program.registers);
    }
}
