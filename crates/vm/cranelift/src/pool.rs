//! Translator context

use crate::Translator;
use cranelift::{codegen::ir::SigRef, prelude::*};

/// Offsets to the memory base
pub mod offsets {
    /// Offset to gas field (after registers)
    pub const GAS_OFFSET: i32 = 8 * (pvm::REGISTER_COUNT as i32);

    /// Offset to memory field
    pub const MEMORY_OFFSET: i32 = GAS_OFFSET + 8;
}

/// Register manager
///
/// Total 16 registers, we actually use memory as the base and then
/// calculate the offsets back to the base.
#[derive(Clone)]
pub struct Pool {
    /// Register values (13 registers)
    ///
    /// [RA, SP, T0, T1, T2, S0, S1, A0, A1, A2, A3, A4, A5]
    pub registers: [Variable; pvm::REGISTER_COUNT],

    /// Current gas value (SSA)
    pub gas: Variable,

    /// The memory pointer
    pub memory: Value,

    /// The VM context pointer
    pub vmctx: Value,

    /// Host call table
    pub call: Call,
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            memory: Value::new(0),
            registers: [Variable::new(0); pvm::REGISTER_COUNT],
            gas: Variable::new(0),
            vmctx: Value::new(0),
            call: Call::default(),
        }
    }
}

/// Host call table
#[derive(Clone)]
pub struct Call {
    /// memory address of call ecalli
    pub ecalli: (SigRef, Value),

    /// memory address of call sbrk
    pub sbrk: (SigRef, Value),

    /// memory address of call mget
    pub mget: (SigRef, Value),

    /// memory address of call mset
    pub mset: (SigRef, Value),
}

impl Default for Call {
    fn default() -> Self {
        Self {
            ecalli: (SigRef::new(0), Value::new(0)),
            sbrk: (SigRef::new(0), Value::new(0)),
            mget: (SigRef::new(0), Value::new(0)),
            mset: (SigRef::new(0), Value::new(0)),
        }
    }
}

impl Translator<'_> {
    /// Init all registers
    ///
    /// NOTE: ignore the costs of the initial memory loads otherwise
    /// we'll have ugly function signatures.
    pub fn init_pool(&mut self, entry: Block, registers: [u64; pvm::REGISTER_COUNT]) -> Value {
        let params = self.builder.block_params(entry).to_vec();
        let [vmctx, pc, table] = [params[0], params[1], params[2]];
        self.pool.vmctx = vmctx;
        self.pool.memory = self.builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            vmctx,
            offsets::MEMORY_OFFSET,
        );

        // init gas
        {
            self.pool.gas = self.builder.declare_var(types::I64);
            let gas = self.builder.ins().load(
                types::I64,
                MemFlags::trusted(),
                vmctx,
                offsets::GAS_OFFSET,
            );
            self.context.builder.def_var(self.context.pool.gas, gas);
        }

        // init registers
        {
            for (i, _reg) in registers.iter().enumerate() {
                let var = self.builder.declare_var(types::I64);
                self.pool.registers[i] = var;
            }
            self.load_registers();
        }

        // init host calls - load function pointers from table
        {
            self.pool.call.ecalli = (
                self.context
                    .builder
                    .import_signature(crate::host::ECALLI.clone()),
                self.context
                    .builder
                    .ins()
                    .load(types::I64, MemFlags::trusted(), table, 0),
            );
            self.pool.call.sbrk = (
                self.context
                    .builder
                    .import_signature(crate::host::SBRK.clone()),
                self.context
                    .builder
                    .ins()
                    .load(types::I64, MemFlags::trusted(), table, 8),
            );
            self.pool.call.mget = (
                self.context
                    .builder
                    .import_signature(crate::host::MGET.clone()),
                self.context
                    .builder
                    .ins()
                    .load(types::I64, MemFlags::trusted(), table, 16),
            );
            self.pool.call.mset = (
                self.context
                    .builder
                    .import_signature(crate::host::MSET.clone()),
                self.context
                    .builder
                    .ins()
                    .load(types::I64, MemFlags::trusted(), table, 24),
            );
        }

        pc
    }

    /// Sync registers to memory
    pub fn sync_registers(&mut self) {
        for i in 0..13 {
            let reg = self.context.builder.use_var(self.context.pool.registers[i]);
            self.context.builder.ins().store(
                MemFlags::trusted(),
                reg,
                self.context.pool.vmctx,
                i as i32 * 8,
            );
        }
    }

    /// Load registers from memory
    pub fn load_registers(&mut self) {
        for i in 0..13 {
            let reg = self.context.builder.ins().load(
                types::I64,
                MemFlags::trusted(),
                self.context.pool.vmctx,
                i as i32 * 8,
            );
            self.context
                .builder
                .def_var(self.context.pool.registers[i], reg);
        }
    }

    /// Load gas from memory
    pub fn load_gas(&mut self) {
        let gas = self.context.builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            self.context.pool.vmctx,
            offsets::GAS_OFFSET,
        );
        self.context.builder.def_var(self.context.pool.gas, gas);
    }

    /// Sync gas to memory
    pub fn store_gas(&mut self) {
        let gas = self.context.builder.use_var(self.context.pool.gas);
        self.context.builder.ins().store(
            MemFlags::trusted(),
            gas,
            self.context.pool.vmctx,
            offsets::GAS_OFFSET,
        );
    }

    /// get register value
    pub fn rget(&mut self, reg: u8) -> Value {
        self.context
            .builder
            .use_var(self.context.pool.registers[reg as usize])
    }

    /// set register value
    pub fn rset(&mut self, reg: u8, value: Value) {
        self.context
            .builder
            .def_var(self.pool.registers[reg as usize], value);
    }
}
