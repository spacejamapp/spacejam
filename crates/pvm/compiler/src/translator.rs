//! Translator module that converts PVM instructions to Cranelift IR

use cranelift::prelude::*;
use parser::{format, Visitor};
use std::collections::HashMap;

/// Temporary visitor wrapper to avoid lifetime issues
pub struct Translator<'a, 'b> {
    pub registers: HashMap<u8, Variable>,
    pub pc: Variable,
    pub memory_ptr: Variable,
    pub execution_mask: Variable,  // Track which execution path we're on
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

        // Declare memory pointer variable (use variable index 14)
        let memory_ptr = Variable::new(14);
        builder.declare_var(memory_ptr, types::I64);

        // Declare execution mask variable (use variable index 15)
        let execution_mask = Variable::new(15);
        builder.declare_var(execution_mask, types::I8);

        Self { registers, pc, memory_ptr, execution_mask, builder }
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

        // Load memory pointer from context.memory_ptr (offset 112 bytes after start)
        let mem_offset = self.builder.ins().iconst(types::I64, 112);
        let mem_addr = self.builder.ins().iadd(context_ptr, mem_offset);
        let mem_ptr_value = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), mem_addr, 0);
        self.builder.def_var(self.memory_ptr, mem_ptr_value);

        // Initialize execution mask to true (all instructions execute initially)
        let true_val = self.builder.ins().iconst(types::I8, 1);
        self.builder.def_var(self.execution_mask, true_val);

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

    fn visit_load_imm_jump(&mut self, format: format::RIO) -> Result<(), Self::Error> {
        let format::RIO { reg0, off0, imm0 } = format;
        // Load immediate value
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);
        
        // Jump by adding offset to current PC
        let current_pc = self.builder.use_var(self.pc);
        let offset_val = self.builder.ins().iconst(types::I64, off0 as i64);
        let target_pc = self.builder.ins().iadd(current_pc, offset_val);
        self.builder.def_var(self.pc, target_pc);
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

    fn visit_rem_u_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend_val);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor_val);

        // Check for division by zero - return dividend if divisor is zero
        let zero = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero);
        
        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let result_32 = self.builder.ins().urem(dividend_32, safe_divisor);
        let result_32_ext = self.builder.ins().sextend(types::I64, result_32);
        
        // Return original dividend for div by zero, otherwise remainder
        let dividend_32_ext = self.builder.ins().sextend(types::I64, dividend_32);
        let result = self.builder.ins().select(is_zero, dividend_32_ext, result_32_ext);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rem_u_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);

        // Check for division by zero - return dividend if divisor is zero
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_val, zero);
        
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor_val);
        let result_rem = self.builder.ins().urem(dividend_val, safe_divisor);
        
        // Return original dividend for div by zero, otherwise remainder
        let result = self.builder.ins().select(is_zero, dividend_val, result_rem);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rem_s_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
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
        
        // Check for overflow (i32::MIN % -1)
        let min_val_32 = self.builder.ins().iconst(types::I32, i32::MIN as i64);
        let neg_one = self.builder.ins().iconst(types::I32, -1);
        let is_min = self.builder.ins().icmp(IntCC::Equal, dividend_32, min_val_32);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor_32, neg_one);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        // Use safe divisor to avoid division faults
        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let safe_divisor = self.builder.ins().select(is_overflow, one_32, safe_divisor);
        let result_32 = self.builder.ins().srem(dividend_32, safe_divisor);
        let result_32_ext = self.builder.ins().sextend(types::I64, result_32);
        
        // Return original dividend for div by zero, 0 for overflow, otherwise remainder
        let dividend_32_ext = self.builder.ins().sextend(types::I64, dividend_32);
        let zero_64 = self.builder.ins().iconst(types::I64, 0);
        let result_or_overflow = self.builder.ins().select(is_overflow, zero_64, result_32_ext);
        let result = self.builder.ins().select(is_zero, dividend_32_ext, result_or_overflow);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rem_s_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend_var = self.registers[&reg0];
        let divisor_var = self.registers[&reg1];
        let dividend_val = self.builder.use_var(dividend_var);
        let divisor_val = self.builder.use_var(divisor_var);

        // Check for division by zero
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_val, zero);
        
        // Check for overflow (i64::MIN % -1)
        let min_val_64 = self.builder.ins().iconst(types::I64, i64::MIN);
        let neg_one = self.builder.ins().iconst(types::I64, -1);
        let is_min = self.builder.ins().icmp(IntCC::Equal, dividend_val, min_val_64);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor_val, neg_one);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        // Use safe divisor to avoid division faults
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor_val);
        let safe_divisor = self.builder.ins().select(is_overflow, one_64, safe_divisor);
        let result_rem = self.builder.ins().srem(dividend_val, safe_divisor);
        
        // Return original dividend for div by zero, 0 for overflow, otherwise remainder  
        let zero_64 = self.builder.ins().iconst(types::I64, 0);
        let result_or_overflow = self.builder.ins().select(is_overflow, zero_64, result_rem);
        let result = self.builder.ins().select(is_zero, dividend_val, result_or_overflow);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shlo_l_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let shift_val = self.builder.ins().ireduce(types::I32, src1_val);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result_32 = self.builder.ins().ishl(src0_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shlo_l_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(src1_val, mask);
        let result = self.builder.ins().ishl(src0_val, safe_shift);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shlo_l_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        
        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().ishl_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shlo_l_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        
        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().ishl_imm(src_val, safe_shift);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shlo_r_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let shift_val = self.builder.ins().ireduce(types::I32, src1_val);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result_32 = self.builder.ins().ushr(src0_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shlo_r_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(src1_val, mask);
        let result = self.builder.ins().ushr(src0_val, safe_shift);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shlo_r_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        
        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().ushr_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shlo_r_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        
        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().ushr_imm(src_val, safe_shift);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shar_r_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let shift_val = self.builder.ins().ireduce(types::I32, src1_val);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result_32 = self.builder.ins().sshr(src0_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shar_r_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(src1_val, mask);
        let result = self.builder.ins().sshr(src0_val, safe_shift);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shar_r_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        
        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().sshr_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shar_r_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        
        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().sshr_imm(src_val, safe_shift);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    // Bit counting operations
    fn visit_leading_zero_bits_64(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        
        let result = self.builder.ins().clz(src_val);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_leading_zero_bits_32(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        
        let result_32 = self.builder.ins().clz(src_32);
        let result_64 = self.builder.ins().uextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_trailing_zero_bits_64(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        
        let result = self.builder.ins().ctz(src_val);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_trailing_zero_bits_32(&mut self, format: format::RR) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        
        let result_32 = self.builder.ins().ctz(src_32);
        let result_64 = self.builder.ins().uextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    // Rotation operations - register variants
    fn visit_rot_l_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_var = self.registers[&reg0];
        let shift_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let shift_val = self.builder.use_var(shift_var);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result = self.builder.ins().rotl(src_val, safe_shift);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rot_l_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_var = self.registers[&reg0];
        let shift_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let shift_val = self.builder.use_var(shift_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_32, mask);
        let result_32 = self.builder.ins().rotl(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_rot_r_64(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_var = self.registers[&reg0];
        let shift_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let shift_val = self.builder.use_var(shift_var);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result = self.builder.ins().rotr(src_val, safe_shift);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rot_r_32(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_var = self.registers[&reg0];
        let shift_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let shift_val = self.builder.use_var(shift_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);
        
        // Mask shift amount to avoid undefined behavior
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_32, mask);
        let result_32 = self.builder.ins().rotr(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    // Rotation operations - immediate variants
    fn visit_rot_r_64_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        
        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().rotr_imm(src_val, safe_shift);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rot_r_32_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        
        // Mask immediate to avoid undefined behavior
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().rotr_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    // Memory load operations - simplified safe implementations
    fn visit_load_u8(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0: _ } = format;
        // For now, just load zero to avoid segfaults
        // TODO: Implement proper memory access with bounds checking
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, zero_val);
        Ok(())
    }

    fn visit_load_u16(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0: _ } = format;
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, zero_val);
        Ok(())
    }

    fn visit_load_u32(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0: _ } = format;
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, zero_val);
        Ok(())
    }

    fn visit_load_u64(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0: _ } = format;
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, zero_val);
        Ok(())
    }

    fn visit_load_i8(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0: _ } = format;
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, zero_val);
        Ok(())
    }

    fn visit_load_i16(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0: _ } = format;
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, zero_val);
        Ok(())
    }

    fn visit_load_i32(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0: _ } = format;
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, zero_val);
        Ok(())
    }

    // Branch operations - currently stubs due to architectural limitations
    fn visit_branch_eq(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        // For now, just implement as no-op to prevent crashes
        // TODO: Implement proper control flow with basic blocks
        Ok(())
    }

    fn visit_branch_eq_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_ne(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_ne_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_lt_u(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_lt_s(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_ge_u(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_ge_s(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_lt_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_lt_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_ge_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_ge_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    // Additional branch operations
    fn visit_branch_gt_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_gt_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_le_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_branch_le_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        Ok(())
    }

    // Jump operations
    fn visit_jump(&mut self, _format: format::O) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_jump_ind(&mut self, _format: format::RI) -> Result<(), Self::Error> {
        Ok(())
    }

    // Conditional move operations
    fn visit_cmov_iz(&mut self, _format: format::RRR) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_cmov_iz_imm(&mut self, _format: format::RRI) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_cmov_nz(&mut self, _format: format::RRR) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_cmov_nz_imm(&mut self, _format: format::RRI) -> Result<(), Self::Error> {
        Ok(())
    }

    // Load immediate and jump indirect operations
    fn visit_load_imm_jump_ind(&mut self, _format: format::RRII) -> Result<(), Self::Error> {
        Ok(())
    }

}
