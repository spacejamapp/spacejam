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
    pub const MEMORY_PTR_OFFSET: i32 = PC_OFFSET + 8;
}

/// Constants pool with Single Static Assignment Values
pub struct Pool {
    /// The context pointer
    pub ctx: Value,

    /// The memory pointer
    pub memory: Value,

    /// ssv for 1
    pub one: Value,
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            ctx: Value::new(0),
            memory: Value::new(0),
            one: Value::new(0),
        }
    }
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
            ctx,
            one: self.builder.ins().iconst(types::I64, 1),
        };

        #[cfg(target_os = "macos")]
        {
            self.memory = program.memory.info.clone();
        }
    }
}
