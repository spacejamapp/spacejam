//! Translator context

use crate::Translator;
use cranelift::prelude::*;
use cranelift_codegen::ir::BlockArg;
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

    /// Register values (13 registers)
    pub registers: [Value; 13],

    /// Current gas value (SSA)
    pub gas: Value,

    /// ssv for 1
    pub one: Value,
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            ctx: Value::new(0),
            memory: Value::new(0),
            registers: [Value::new(0); 13],
            gas: Value::new(0),
            one: Value::new(0),
        }
    }
}

impl Translator<'_> {
    /// Initialize context
    pub fn init_context(&mut self, program: &Program, ctx: Value) {
        tracing::debug!("memory info: {:?}", program.memory.info);
        self.pool.memory = self.builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            ctx,
            offsets::MEMORY_PTR_OFFSET,
        );
        self.pool.ctx = ctx;
        self.pool.one = self.builder.ins().iconst(types::I64, 1);

        #[cfg(target_os = "macos")]
        {
            self.memory = program.memory.info.clone();
        }
    }

    /// load registers from the context
    pub fn load_registers(&mut self) {
        for i in 0..13 {
            self.pool.registers[i] = self.builder.ins().load(
                types::I64,
                MemFlags::trusted(),
                self.pool.ctx,
                i as i32 * 8,
            );
        }
    }

    /// Get register and gas values as block parameters (14 total)
    pub fn args(&self) -> Vec<BlockArg> {
        let mut params = Vec::new();
        for &reg in &self.pool.registers {
            params.push(BlockArg::Value(reg));
        }
        // Add gas as the 14th parameter
        params.push(BlockArg::Value(self.pool.gas));
        params
    }

    /// Update registers and gas from block parameters (14 total)
    pub fn params(&mut self, params: &[Value]) {
        self.pool.registers.copy_from_slice(&params[..13]);
        self.pool.gas = params[13];
    }

    /// Sync registers to memory
    pub fn sync_params(&mut self) {
        self.sync_registers();
        self.sync_gas();
    }

    /// Load parameters from memory
    pub fn load_params(&mut self) {
        self.load_registers();
        self.load_gas();
    }

    /// Sync registers to memory
    pub fn sync_registers(&mut self) {
        for i in 0..13 {
            self.builder.ins().store(
                MemFlags::trusted(),
                self.pool.registers[i],
                self.pool.ctx,
                i as i32 * 8,
            );
        }
    }

    /// Sync gas to memory
    pub fn sync_gas(&mut self) {
        self.builder.ins().store(
            MemFlags::trusted(),
            self.pool.gas,
            self.pool.ctx,
            offsets::GAS_OFFSET,
        );
    }

    /// Load gas from memory into SSA value
    pub fn load_gas(&mut self) {
        self.pool.gas = self.builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            self.pool.ctx,
            offsets::GAS_OFFSET,
        );
    }
}
