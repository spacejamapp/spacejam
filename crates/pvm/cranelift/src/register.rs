//! Register related operations

use crate::Translator;
use cranelift::prelude::*;

impl Translator<'_> {
    /// Initialize registers from context
    pub fn init_registers(&mut self, registers: &[u64; pvm::REGISTER_COUNT]) {
        for (i, reg) in registers.iter().enumerate() {
            let var = Variable::new(i);
            self.builder.declare_var(var, types::I64);
            let val = self.builder.ins().iconst(types::I64, *reg as i64);
            self.builder.def_var(var, val);
            self.registers.insert(i as u8, var);
        }
    }

    /// get register value
    pub fn rget(&mut self, reg: u8) -> Value {
        let reg_var = self.registers[&reg];
        self.builder.use_var(reg_var)
    }

    /// set register value
    pub fn rset(&mut self, reg: u8, value: Value) {
        let reg_var = self.registers[&reg];
        self.builder.def_var(reg_var, value);
    }

    // Save registers to context
    pub fn save_registers(&mut self) {
        for i in 0..self.registers.len() {
            let var = self.registers[&(i as u8)];
            let val = self.builder.use_var(var);
            self.builder
                .ins()
                .store(MemFlags::new(), val, self.ctx, (i * 8) as i32);
        }
    }

    /// Load registers from context
    pub fn load_registers(&mut self) {
        for i in 0..self.registers.len() {
            let var = self.registers[&(i as u8)];
            let val =
                self.builder
                    .ins()
                    .load(types::I64, MemFlags::trusted(), self.ctx, (i * 8) as i32);
            self.builder.def_var(var, val);
        }
    }
}
