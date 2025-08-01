//! Translator module that converts PVM instructions to Cranelift IR

use cranelift::prelude::*;
use parser::{format, Visitor};
use std::collections::HashMap;

/// Temporary visitor wrapper to avoid lifetime issues
pub struct Translator<'a, 'b> {
    pub registers: HashMap<u8, Variable>,
    pub pc: Variable,
    builder: &'a mut FunctionBuilder<'b>,
}
 
impl<'a, 'b> Translator<'a, 'b> {
    /// Create a new translator with PVM register variables and PC
    pub fn new(builder: &'a mut FunctionBuilder<'b>) -> Self {
        let mut registers = HashMap::new();

        // Declare all 13 PVM registers as Cranelift variables
        // PVM has 13 registers: ra(0), sp(1), unused(2,3,4), s0-s1(5-6), a0-a4(7-11), unused(12)
        for i in 0..13 {
            let var = Variable::new(i);
            builder.declare_var(var, types::I64);
            registers.insert(i as u8, var);
        }

        // Declare PC variable (use variable index 13)
        let pc = Variable::new(13);
        builder.declare_var(pc, types::I64);

        Self { registers, pc, builder }
    }

    /// Load initial execution context (registers + PC) from memory pointer
    pub fn load_initial_context(&mut self, context_ptr: Value) -> Result<(), anyhow::Error> {
        // Load all 13 registers from context.registers
        for i in 0..13 {
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(context_ptr, offset);
            let value = self
                .builder
                .ins()
                .load(types::I64, MemFlags::new(), addr, 0);
            let var = self.registers[&(i as u8)];
            self.builder.def_var(var, value);
        }

        // Load PC from context.pc (offset 13 * 8 = 104 bytes after start)
        let pc_offset = self.builder.ins().iconst(types::I64, 104);
        let pc_addr = self.builder.ins().iadd(context_ptr, pc_offset);
        let pc_value = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), pc_addr, 0);
        self.builder.def_var(self.pc, pc_value);

        Ok(())
    }

    /// Translate a PVM program to Cranelift IR and return final context values
    pub fn translate(&mut self, program: &[u8]) -> Result<(Vec<Value>, Value), anyhow::Error> {
        let blob = parser::program::deblob(program)?;
        let mut reader = blob.reader();

        while !reader.eof() {
            let instruction_offset = reader.read()?;
            let instruction = instruction_offset.value;
            
            // Increment PC by instruction size before executing instruction
            let current_pc = self.builder.use_var(self.pc);
            let instruction_size = self.builder.ins().iconst(types::I64, instruction_offset.range.len() as i64);
            let new_pc = self.builder.ins().iadd(current_pc, instruction_size);
            self.builder.def_var(self.pc, new_pc);
            
            self.visit(instruction)?;
        }

        // Return all 13 register values + PC
        let mut register_values = Vec::with_capacity(13);
        for i in 0..13 {
            let var = self.registers[&(i as u8)];
            register_values.push(self.builder.use_var(var));
        }
        
        let pc_value = self.builder.use_var(self.pc);

        Ok((register_values, pc_value))
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
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);
        Ok(())
    }

    fn visit_load_imm_64(&mut self, format: format::REI) -> Result<(), Self::Error> {
        let format::REI { reg0, eimm0 } = format;
        let imm_val = self.builder.ins().iconst(types::I64, eimm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);
        Ok(())
    }

    fn visit_add_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if reg0 >= 13 || reg1 >= 13 {
            anyhow::bail!("Invalid register numbers: dst={}, src={}", reg0, reg1);
        }

        // Load source register, truncate to 32-bit, add immediate, sign extend to 64-bit
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().iadd(src_32, imm_val);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_add_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if reg0 >= 13 || reg1 >= 13 {
            anyhow::bail!("Invalid register numbers: dst={}, src={}", reg0, reg1);  
        }

        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().iadd(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_add_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let src1_32 = self.builder.ins().ireduce(types::I32, src1_val);
        let result_32 = self.builder.ins().iadd(src0_32, src1_32);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_add_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().iadd(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_sub_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let src1_32 = self.builder.ins().ireduce(types::I32, src1_val);
        let result_32 = self.builder.ins().isub(src0_32, src1_32);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_sub_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().isub(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_mul_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let src1_32 = self.builder.ins().ireduce(types::I32, src1_val);
        let result_32 = self.builder.ins().imul(src0_32, src1_32);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_mul_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().imul(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_mul_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().imul(src_32, imm_val);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_mul_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().imul(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_u_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend_val);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor_val);

        // Check for division by zero and return u64::MAX if so
        let zero = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero);
        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        
        // Use conditional blocks to avoid division by zero
        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let result_32 = self.builder.ins().udiv(dividend_32, safe_divisor);
        let result_32_ext = self.builder.ins().uextend(types::I64, result_32);
        let result = self.builder.ins().select(is_zero, max_val, result_32_ext);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_u_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if reg0 >= 13 || reg1 >= 13 || reg2 >= 13 {
            anyhow::bail!("Invalid register numbers: {}, {}, {}, ", reg0, reg1, reg2);
        }

        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);

        // Check for division by zero and return u64::MAX if so
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_val, zero);
        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        
        // Use conditional blocks to avoid division by zero
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor_val);
        let result_div = self.builder.ins().udiv(dividend_val, safe_divisor);
        let result = self.builder.ins().select(is_zero, max_val, result_div);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_s_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend_val);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor_val);

        // Check for division by zero
        let zero = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero);
        
        // Check for overflow (i32::MIN / -1)
        let min_val_32 = self.builder.ins().iconst(types::I32, i32::MIN as i64);
        let neg_one = self.builder.ins().iconst(types::I32, -1);
        let is_min = self.builder.ins().icmp(IntCC::Equal, dividend_32, min_val_32);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor_32, neg_one);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        let min_result = self.builder.ins().iconst(types::I64, i32::MIN as i64);
        
        // Use safe divisor to avoid division faults
        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let safe_divisor = self.builder.ins().select(is_overflow, one_32, safe_divisor);
        let result_32 = self.builder.ins().sdiv(dividend_32, safe_divisor);
        let result_32_ext = self.builder.ins().sextend(types::I64, result_32);
        
        // Return u64::MAX for div by zero, i32::MIN for overflow, otherwise result
        let result_or_overflow = self.builder.ins().select(is_overflow, min_result, result_32_ext);
        let result = self.builder.ins().select(is_zero, max_val, result_or_overflow);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_s_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);

        // Check for division by zero
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_val, zero);
        
        // Check for overflow (i64::MIN / -1)
        let min_val_64 = self.builder.ins().iconst(types::I64, i64::MIN);
        let neg_one = self.builder.ins().iconst(types::I64, -1);
        let is_min = self.builder.ins().icmp(IntCC::Equal, dividend_val, min_val_64);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor_val, neg_one);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        
        // Use safe divisor to avoid division faults
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor_val);
        let safe_divisor = self.builder.ins().select(is_overflow, one_64, safe_divisor);
        let result_div = self.builder.ins().sdiv(dividend_val, safe_divisor);
        
        // Return u64::MAX for div by zero, original dividend for overflow, otherwise result
        let result_or_overflow = self.builder.ins().select(is_overflow, dividend_val, result_div);
        let result = self.builder.ins().select(is_zero, max_val, result_or_overflow);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_move_reg(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, src_val);
        Ok(())
    }

    fn visit_and(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().band(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_and_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().band(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_or(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().bor(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_or_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().bor(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_xor(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let result = self.builder.ins().bxor(src0_val, src1_val);
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_xor_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().bxor(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

}
