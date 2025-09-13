//! Translator context

use crate::Translator;
use cranelift::prelude::*;

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

    /// Offset to memory field
    pub const MEMORY_OFFSET: i32 = GAS_OFFSET + 8;
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
    pub registers: [Variable; 13],

    /// Current gas value (SSA)
    pub gas: Variable,

    /// The memory pointer
    pub memory: Value,

    /// The VM context pointer
    pub vmctx: Value,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            memory: Value::new(0),
            registers: [Variable::new(0); 13],
            gas: Variable::new(0),
            vmctx: Value::new(0),
        }
    }
}

impl Translator<'_> {
    /// Init all registers
    ///
    /// NOTE: ignore the costs of the initial memory loads otherwise
    /// we'll have ugly function signatures.
    pub fn init_registers(&mut self, entry: Block) -> Value {
        let params = self.builder.block_params(entry).to_vec();
        let [vmctx, pc] = [params[0], params[1]];
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
        for i in 0..13 {
            let var = self.builder.declare_var(types::I64);
            let val = self
                .builder
                .ins()
                .load(types::I64, MemFlags::trusted(), vmctx, i as i32 * 8);
            self.builder.def_var(var, val);
            self.pool.registers[i] = var;
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
