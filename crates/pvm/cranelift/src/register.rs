//! Translator context

use crate::Translator;
use cranelift::prelude::*;
use cranelift_codegen::ir::BlockArg;

/// Offsets to the memory base
///
/// Context {
///   registers: [u64; 13],
///   gas: i64,
///   dispatch: [u64; pvm::MAX_FUNCTIONS],
///   memory: *mut u8,
///   inner_ctx: *mut u8
/// }
///
/// For calculating the offsets, we use the following formula:
///
/// dispatch_start = memory_ptr - DISPATCH_OFFSET
/// gas_start = memory_ptr - GAS_OFFSET
/// ctx_start = registers_start = memory_ptr - REGISTERS_OFFSET
pub mod offsets {

    /// Offset to gas field (after registers)
    pub const GAS_OFFSET: i32 = 8 * (pvm::REGISTER_COUNT as i32);

    /// Offset to dispatch table
    pub const DISPATCH_OFFSET: i32 = GAS_OFFSET + 8;

    /// Offset to memory field
    pub const MEMORY_OFFSET: i32 = DISPATCH_OFFSET + 8 * (pvm::MAX_FUNCTIONS as i32);
}

/// Register manager
///
/// Total 16 registers, we actually use memory as the base and then
/// calculate the offsets back to the base.
#[derive(Clone)]
pub struct Registers {
    /// Register values (13 registers)
    ///
    /// [RA, SP, T0, T1, T2, S0, S1, A0, A1, A2, A3, A4, A5]
    pub registers: [Value; 13],

    /// Current gas value (SSA)
    pub gas: Value,

    /// The memory pointer
    pub memory: Value,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            memory: Value::new(0),
            registers: [Value::new(0); 13],
            gas: Value::new(0),
        }
    }
}

impl Translator<'_> {
    /// Load dispatch table pointer to register
    pub fn dispatch(&mut self, index: Value) -> Value {
        let mut target = self
            .builder
            .ins()
            .iadd_imm(self.pool.memory, -offsets::DISPATCH_OFFSET as i64);
        let offset = self.builder.ins().imul_imm(index, 8);
        target = self.builder.ins().iadd(target, offset);
        target
    }

    /// Load ctx pointer to register
    pub fn ctx(&mut self) -> Value {
        self.builder
            .ins()
            .iadd_imm(self.pool.memory, -offsets::MEMORY_OFFSET as i64)
    }

    /// load registers from the context
    ///
    /// TODO: fix the offsets, also, we don't need to load all of
    /// the registers
    pub fn load_registers(&mut self) {
        for i in 0..13 {
            self.pool.registers[i] = self.builder.ins().load(
                types::I64,
                MemFlags::trusted(),
                self.pool.memory,
                i as i32 * 8 - offsets::MEMORY_OFFSET as i32,
            );
        }
    }

    /// Sync registers to memory
    pub fn sync_registers(&mut self) {
        for i in 0..13 {
            self.builder.ins().store(
                MemFlags::trusted(),
                self.pool.registers[i],
                self.pool.memory,
                i as i32 * 8 - offsets::MEMORY_OFFSET as i32,
            );
        }
    }

    /// Sync gas to memory
    pub fn store_gas(&mut self) {
        self.builder.ins().store(
            MemFlags::trusted(),
            self.pool.gas,
            self.pool.memory,
            -offsets::GAS_OFFSET as i32,
        );
    }

    /// Load gas from memory into SSA value
    pub fn load_gas(&mut self) {
        self.pool.gas = self.builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            self.pool.memory,
            -offsets::GAS_OFFSET as i32,
        );
    }

    /// Get function arguments
    pub fn args(&self) -> Vec<Value> {
        vec![self.pool.registers[..13].to_vec(), vec![self.pool.gas]].concat()
    }

    /// get block arguments
    pub fn block_args(&self) -> Vec<BlockArg> {
        self.args().iter().map(|v| BlockArg::Value(*v)).collect()
    }

    /// load block arguments
    pub fn load_block_args(&mut self, block: Block) {
        let args = self.builder.block_params(block);
        for i in 0..13 {
            self.pool.registers[i] = args[i];
        }
        self.pool.gas = args[13];
    }

    /// get register value
    pub fn rget(&mut self, reg: u8) -> Value {
        self.pool.registers[reg as usize]
    }

    /// set register value
    pub fn rset(&mut self, reg: u8, value: Value) {
        self.pool.registers[reg as usize] = value;
    }
}
