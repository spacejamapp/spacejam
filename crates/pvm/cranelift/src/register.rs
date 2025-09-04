//! Translator context

use crate::Translator;
use cranelift::prelude::*;

/// ExtendedContext memory layout offsets
pub mod offsets {
    /// Offset to gas field (after registers)
    pub const GAS_OFFSET: i32 = -8;

    /// Size of register array in bytes
    pub const REGISTERS_OFFSET: i32 = -8 * (pvm::REGISTER_COUNT as i32) + GAS_OFFSET;
}

/// Register manager
///
/// Total 16 registers, we actually use memory as the base and then
/// calculate the offsets back to the base.
#[derive(Clone)]
pub struct Registers {
    /// Register values (13 registers)
    pub registers: [Value; 13],

    /// Current gas value (SSA)
    pub gas: Value,

    /// The memory pointer
    pub memory: Value,

    /// The dispatch table size in bits
    ///
    /// used for calculating the dispatch table address
    pub dispatch: i32,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            memory: Value::new(0),
            registers: [Value::new(0); 13],
            gas: Value::new(0),
            dispatch: 0,
        }
    }
}

impl Translator<'_> {
    /// Load dispatch table pointer to register
    pub fn dispatch(&mut self, reg: &mut Value) {
        *reg = self
            .builder
            .ins()
            .iadd_imm(self.pool.memory, self.pool.dispatch as i64);
    }

    /// Load ctx pointer to register
    pub fn ctx(&mut self) -> Value {
        self.builder.ins().iadd_imm(
            self.pool.memory,
            (self.pool.dispatch + offsets::GAS_OFFSET) as i64,
        )
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
                i as i32 * 8,
            );
        }
    }

    /// Sync registers to memory
    ///
    /// TODO: fix the offsets, also, we don't need to sync all of
    /// the registers
    pub fn sync_registers(&mut self) {
        for i in 0..13 {
            self.builder.ins().store(
                MemFlags::trusted(),
                self.pool.registers[i],
                self.pool.memory,
                i as i32 * 8,
            );
        }
    }

    /// Sync gas to memory
    pub fn store_gas(&mut self) {
        self.builder.ins().store(
            MemFlags::trusted(),
            self.pool.gas,
            self.pool.memory,
            -(self.pool.dispatch + offsets::GAS_OFFSET) as i32,
        );
    }

    /// Load gas from memory into SSA value
    pub fn load_gas(&mut self) {
        self.pool.gas = self.builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            self.pool.memory,
            self.pool.dispatch + offsets::GAS_OFFSET as i32,
        );
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
