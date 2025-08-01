//! Translator module that converts PVM instructions to Cranelift IR

use cranelift::prelude::*;
use parser::{format, Visitor};
use std::collections::HashMap;

/// Temporary visitor wrapper to avoid lifetime issues
pub struct Translator<'a, 'b> {
    pub registers: HashMap<u8, Variable>,
    builder: &'a mut FunctionBuilder<'b>,
}

impl<'a, 'b> Translator<'a, 'b> {
    /// Create a new translator with PVM register variables
    pub fn new(builder: &'a mut FunctionBuilder<'b>) -> Self {
        let mut registers = HashMap::new();

        // Declare all 13 PVM registers as Cranelift variables
        // PVM has 13 registers: ra(0), sp(1), unused(2,3,4), s0-s1(5-6), a0-a4(7-11), unused(12)
        for i in 0..13 {
            let var = Variable::new(i);
            builder.declare_var(var, types::I64);
            registers.insert(i as u8, var);
        }

        Self { registers, builder }
    }

    /// Load initial register values from memory pointer
    pub fn load_initial_registers(&mut self, registers_ptr: Value) -> Result<(), anyhow::Error> {
        for i in 0..13 {
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(registers_ptr, offset);
            let value = self
                .builder
                .ins()
                .load(types::I64, MemFlags::new(), addr, 0);
            let var = self.registers[&(i as u8)];
            self.builder.def_var(var, value);
        }
        Ok(())
    }

    /// Translate a PVM program to Cranelift IR
    pub fn translate(&mut self, program: &[u8]) -> Result<Vec<Value>, anyhow::Error> {
        let blob = parser::program::deblob(program)?;
        let mut reader = blob.reader();

        while !reader.eof() {
            let instruction_offset = reader.read()?;
            let instruction = instruction_offset.value;
            self.visit(instruction)?;
        }

        // Return all 13 register values
        let mut register_values = Vec::with_capacity(13);
        for i in 0..13 {
            let var = self.registers[&(i as u8)];
            register_values.push(self.builder.use_var(var));
        }

        Ok(register_values)
    }
}

impl Visitor for Translator<'_, '_> {
    type Error = anyhow::Error;

    fn visit_trap(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_fallthrough(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_load_imm(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        if reg0 >= 13 {
            anyhow::bail!("Invalid register number: {}", reg0);
        }

        // Load immediate value into register variable
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);
        Ok(())
    }

    fn visit_add_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if reg0 >= 13 || reg1 >= 13 {
            anyhow::bail!("Invalid register numbers: dst={}, src={}", reg0, reg1);
        }

        // Load source register, truncate to 32-bit, add immediate, zero-extend
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().iadd(src_32, imm_val);
        let result_64 = self.builder.ins().uextend(types::I64, result_32);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }
}
