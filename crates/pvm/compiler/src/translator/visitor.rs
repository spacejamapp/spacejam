//! Visitor implementation for PVM instructions

use crate::{translator::memory::MemorySize, Translator};
use cranelift::prelude::*;
use parser::{format, Visitor};

impl Visitor for Translator<'_, '_> {
    type Error = anyhow::Error;

    fn visit_trap(&mut self) -> Result<(), Self::Error> {
        // Trap instruction should preserve PC (don't modify it)
        // The PC already points to the trap instruction location
        Ok(())
    }

    fn visit_fallthrough(&mut self) -> Result<(), Self::Error> {
        // Fallthrough instruction sets PC to 0 for normal halt (ret_halt test expects PC=0)
        let halt_pc = self.builder.ins().iconst(types::I64, 0);
        self.builder.def_var(self.pc, halt_pc);
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
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_32, min_val_32);
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
        let result_or_overflow = self
            .builder
            .ins()
            .select(is_overflow, min_result, result_32_ext);
        let result = self
            .builder
            .ins()
            .select(is_zero, max_val, result_or_overflow);

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
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_val, min_val_64);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor_val, neg_one);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);

        // Use safe divisor to avoid division faults
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor_val);
        let safe_divisor = self.builder.ins().select(is_overflow, one_64, safe_divisor);
        let result_div = self.builder.ins().sdiv(dividend_val, safe_divisor);

        // Return u64::MAX for div by zero, original dividend for overflow, otherwise result
        let result_or_overflow = self
            .builder
            .ins()
            .select(is_overflow, dividend_val, result_div);
        let result = self
            .builder
            .ins()
            .select(is_zero, max_val, result_or_overflow);

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
        let result = self
            .builder
            .ins()
            .select(is_zero, dividend_32_ext, result_32_ext);

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
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_32, min_val_32);
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
        let result_or_overflow = self
            .builder
            .ins()
            .select(is_overflow, zero_64, result_32_ext);
        let result = self
            .builder
            .ins()
            .select(is_zero, dividend_32_ext, result_or_overflow);

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
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_val, min_val_64);
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
        let result = self
            .builder
            .ins()
            .select(is_zero, dividend_val, result_or_overflow);

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

    // Memory load operations
    fn visit_load_u8(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory read
        let value = self.emit_memory_read(address, MemorySize::Byte);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_u16(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory read
        let value = self.emit_memory_read(address, MemorySize::Word);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_u32(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory read
        let value = self.emit_memory_read(address, MemorySize::DWord);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_u64(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory read
        let value = self.emit_memory_read(address, MemorySize::QWord);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_i8(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(address, MemorySize::Byte);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_i16(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(address, MemorySize::Word);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_i32(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(address, MemorySize::DWord);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    // Indirect load operations
    fn visit_load_ind_u8(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        let value = self.emit_memory_read(effective_addr, MemorySize::Byte);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_ind_u16(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        let value = self.emit_memory_read(effective_addr, MemorySize::Word);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_ind_u32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        let value = self.emit_memory_read(effective_addr, MemorySize::DWord);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_ind_u64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        let value = self.emit_memory_read(effective_addr, MemorySize::QWord);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_ind_i8(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(effective_addr, MemorySize::Byte);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_ind_i16(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(effective_addr, MemorySize::Word);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_ind_i32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit signed memory read
        let value = self.emit_memory_read_signed(effective_addr, MemorySize::DWord);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    // Branch operations - for linear execution, they're no-ops that terminate blocks
    fn visit_branch_eq(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        // Branch instructions should be handled by visit_with_control_flow
        // If we reach here, it means we're in linear execution mode (no branches detected)
        // This shouldn't happen for programs with branches, so this is likely an error
        eprintln!("ERROR: visit_branch_eq called in linear mode - this indicates control flow detection failed");
        Ok(())
    }

    fn visit_branch_eq_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ne(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ne_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_lt_u(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_lt_s(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ge_u(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ge_s(&mut self, _format: format::RRO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_lt_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_lt_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ge_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_ge_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    // Additional branch operations
    fn visit_branch_gt_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_gt_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_le_u_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_branch_le_s_imm(&mut self, _format: format::RIO) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    // Jump operations
    fn visit_jump(&mut self, _format: format::O) -> Result<(), Self::Error> {
        self.builder.ins().return_(&[]);
        Ok(())
    }

    fn visit_jump_ind(&mut self, format: format::RI) -> Result<(), Self::Error> {
        // Indirect jump: PC = reg0 + immediate (following interpreter logic)
        let reg0_var = self.registers[&format.reg0];
        let reg0_val = self.builder.use_var(reg0_var);
        let offset = self.builder.ins().iconst(types::I64, format.imm0 as i64);
        let target_addr = self.builder.ins().iadd(reg0_val, offset);

        // Set PC to the computed target address
        self.builder.def_var(self.pc, target_addr);

        // For indirect jumps, we need to save state and return to runtime
        // Get the context pointer parameter from entry block
        let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
        let context_ptr = self.builder.block_params(entry_block)[0];

        // Store all 13 register values back to context.registers
        for i in 0..13 {
            let reg_var = self.registers[&(i as u8)];
            let reg_value = self.builder.use_var(reg_var);
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(context_ptr, offset);
            self.builder
                .ins()
                .store(MemFlags::new(), reg_value, addr, 0);
        }

        // Store PC back to context.pc (offset 104)
        let pc_value = self.builder.use_var(self.pc);
        let pc_offset = self.builder.ins().iconst(types::I64, 104);
        let pc_addr = self.builder.ins().iadd(context_ptr, pc_offset);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_value, pc_addr, 0);

        self.builder.ins().return_(&[]);
        Ok(())
    }

    // Conditional move operations
    fn visit_cmov_iz(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        // Conditional move if zero: if reg1 == 0, reg2 = reg0 (following interpreter logic)
        let reg0_var = self.registers[&format.reg0];
        let reg1_var = self.registers[&format.reg1];
        let reg2_var = self.registers[&format.reg2];

        let reg0_val = self.builder.use_var(reg0_var); // source value
        let reg1_val = self.builder.use_var(reg1_var); // condition value
        let reg2_val = self.builder.use_var(reg2_var); // destination current value

        // Check if reg1 is zero (condition register)
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, reg1_val, zero);

        // Select between reg0 (if condition met) or current reg2 value (if condition not met)
        let new_val = self.builder.ins().select(is_zero, reg0_val, reg2_val);
        self.builder.def_var(reg2_var, new_val);

        Ok(())
    }

    fn visit_cmov_iz_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        // Conditional move immediate if zero: if reg1 == 0, reg0 = imm
        let reg0_var = self.registers[&format.reg0];
        let reg1_var = self.registers[&format.reg1];

        let reg1_val = self.builder.use_var(reg1_var);
        let reg0_val = self.builder.use_var(reg0_var);
        let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);

        // Check if reg1 is zero
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, reg1_val, zero);

        // Select between immediate (if zero) or current reg0 value (if not zero)
        let new_val = self.builder.ins().select(is_zero, imm_val, reg0_val);
        self.builder.def_var(reg0_var, new_val);

        Ok(())
    }

    fn visit_cmov_nz(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        // Conditional move if not zero: if reg1 != 0, reg2 = reg0 (following interpreter logic)
        let reg0_var = self.registers[&format.reg0];
        let reg1_var = self.registers[&format.reg1];
        let reg2_var = self.registers[&format.reg2];

        let reg0_val = self.builder.use_var(reg0_var); // source value
        let reg1_val = self.builder.use_var(reg1_var); // condition value
        let reg2_val = self.builder.use_var(reg2_var); // destination current value

        // Check if reg1 is not zero (condition register)
        let zero = self.builder.ins().iconst(types::I64, 0);
        let not_zero = self.builder.ins().icmp(IntCC::NotEqual, reg1_val, zero);

        // Select between reg0 (if condition met) or current reg2 value (if condition not met)
        let new_val = self.builder.ins().select(not_zero, reg0_val, reg2_val);
        self.builder.def_var(reg2_var, new_val);

        Ok(())
    }

    fn visit_cmov_nz_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        // Conditional move immediate if not zero: if reg1 != 0, reg0 = imm
        let reg0_var = self.registers[&format.reg0];
        let reg1_var = self.registers[&format.reg1];

        let reg1_val = self.builder.use_var(reg1_var);
        let reg0_val = self.builder.use_var(reg0_var);
        let imm_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);

        // Check if reg1 is not zero
        let zero = self.builder.ins().iconst(types::I64, 0);
        let not_zero = self.builder.ins().icmp(IntCC::NotEqual, reg1_val, zero);

        // Select between immediate (if not zero) or current reg0 value (if zero)
        let new_val = self.builder.ins().select(not_zero, imm_val, reg0_val);
        self.builder.def_var(reg0_var, new_val);

        Ok(())
    }

    // Load immediate and jump indirect operations
    fn visit_load_imm_jump_ind(&mut self, format: format::RRII) -> Result<(), Self::Error> {
        // Load immediate into first register and jump indirect to second register + immediate
        // Following interpreter logic: rset(reg0, imm0); djump(rget(reg1) + imm1)
        let reg0_var = self.registers[&format.reg0];
        let imm0_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
        self.builder.def_var(reg0_var, imm0_val);

        // Then compute jump target: reg1 + imm1
        let reg1_var = self.registers[&format.reg1];
        let reg1_val = self.builder.use_var(reg1_var);
        let imm1_val = self.builder.ins().iconst(types::I64, format.imm1 as i64);
        let target_addr = self.builder.ins().iadd(reg1_val, imm1_val);

        // Set PC to the computed target address
        self.builder.def_var(self.pc, target_addr);

        // For indirect jumps, we need to save state and return to runtime
        // Get the context pointer parameter from entry block
        let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
        let context_ptr = self.builder.block_params(entry_block)[0];

        // Store all 13 register values back to context.registers
        for i in 0..13 {
            let reg_var = self.registers[&(i as u8)];
            let reg_value = self.builder.use_var(reg_var);
            let offset = self.builder.ins().iconst(types::I64, (i * 8) as i64);
            let addr = self.builder.ins().iadd(context_ptr, offset);
            self.builder
                .ins()
                .store(MemFlags::new(), reg_value, addr, 0);
        }

        // Store PC back to context.pc (offset 104)
        let pc_value = self.builder.use_var(self.pc);
        let pc_offset = self.builder.ins().iconst(types::I64, 104);
        let pc_addr = self.builder.ins().iadd(context_ptr, pc_offset);
        self.builder
            .ins()
            .store(MemFlags::new(), pc_value, pc_addr, 0);

        self.builder.ins().return_(&[]);
        Ok(())
    }

    // Store operations
    fn visit_store_u8(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Truncate to 8 bits
        let truncated = self.builder.ins().ireduce(types::I8, src_val);

        // Emit memory write
        self.emit_memory_write(address, truncated, MemorySize::Byte);

        Ok(())
    }

    fn visit_store_u16(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Truncate to 16 bits
        let truncated = self.builder.ins().ireduce(types::I16, src_val);

        // Emit memory write
        self.emit_memory_write(address, truncated, MemorySize::Word);

        Ok(())
    }

    fn visit_store_u32(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Truncate to 32 bits
        let truncated = self.builder.ins().ireduce(types::I32, src_val);

        // Emit memory write
        self.emit_memory_write(address, truncated, MemorySize::DWord);

        Ok(())
    }

    fn visit_store_u64(&mut self, format: format::RI) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory write
        self.emit_memory_write(address, src_val, MemorySize::QWord);

        Ok(())
    }

    // Store immediate operations
    fn visit_store_imm_u8(&mut self, format: format::II) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I8, (imm1 as u8) as i64);

        // Emit memory write
        self.emit_memory_write(address, value, MemorySize::Byte);

        Ok(())
    }

    fn visit_store_imm_u16(&mut self, format: format::II) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I16, (imm1 as u16) as i64);

        // Emit memory write
        self.emit_memory_write(address, value, MemorySize::Word);

        Ok(())
    }

    fn visit_store_imm_u32(&mut self, format: format::II) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I32, (imm1 as u32) as i64);

        // Emit memory write
        self.emit_memory_write(address, value, MemorySize::DWord);

        Ok(())
    }

    fn visit_store_imm_u64(&mut self, format: format::II) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I64, imm1 as i64);

        // Emit memory write
        self.emit_memory_write(address, value, MemorySize::QWord);

        Ok(())
    }

    // Indirect store operations
    fn visit_store_ind_u8(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Truncate to 8 bits
        let truncated = self.builder.ins().ireduce(types::I8, src_val);

        // Emit memory write
        self.emit_memory_write(effective_addr, truncated, MemorySize::Byte);

        Ok(())
    }

    fn visit_store_ind_u16(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Truncate to 16 bits
        let truncated = self.builder.ins().ireduce(types::I16, src_val);

        // Emit memory write
        self.emit_memory_write(effective_addr, truncated, MemorySize::Word);

        Ok(())
    }

    fn visit_store_ind_u32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Truncate to 32 bits
        let truncated = self.builder.ins().ireduce(types::I32, src_val);

        // Emit memory write
        self.emit_memory_write(effective_addr, truncated, MemorySize::DWord);

        Ok(())
    }

    fn visit_store_ind_u64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory write
        self.emit_memory_write(effective_addr, src_val, MemorySize::QWord);

        Ok(())
    }

    // Store immediate indirect operations
    fn visit_store_imm_ind_u8(&mut self, format: format::RII) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I8, (imm1 as u8) as i64);

        // Emit memory write
        self.emit_memory_write(effective_addr, value, MemorySize::Byte);

        Ok(())
    }

    fn visit_store_imm_ind_u16(&mut self, format: format::RII) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I16, (imm1 as u16) as i64);

        // Emit memory write
        self.emit_memory_write(effective_addr, value, MemorySize::Word);

        Ok(())
    }

    fn visit_store_imm_ind_u32(&mut self, format: format::RII) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I32, (imm1 as u32) as i64);

        // Emit memory write
        self.emit_memory_write(effective_addr, value, MemorySize::DWord);

        Ok(())
    }

    fn visit_store_imm_ind_u64(&mut self, format: format::RII) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I64, imm1 as i64);

        // Emit memory write
        self.emit_memory_write(effective_addr, value, MemorySize::QWord);

        Ok(())
    }

    // === MISSING INSTRUCTION IMPLEMENTATIONS ===

    // Negate and add immediate instructions
    fn visit_neg_add_imm_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        
        // Negate the source and add immediate: -src + imm
        let negated = self.builder.ins().ineg(src_32);
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().iadd(negated, imm_val);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_neg_add_imm_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        
        // Negate the source and add immediate: -src + imm  
        let negated = self.builder.ins().ineg(src_val);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().iadd(negated, imm_val);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    // Set comparison instructions (register variants)
    fn visit_set_lt_u(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        
        // Compare: reg0 < reg1 (unsigned)
        let is_less = self.builder.ins().icmp(IntCC::UnsignedLessThan, src0_val, src1_val);
        let result = self.builder.ins().uextend(types::I64, is_less);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_set_lt_s(&mut self, format: format::RRR) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);
        
        // Compare: reg0 < reg1 (signed)
        let is_less = self.builder.ins().icmp(IntCC::SignedLessThan, src0_val, src1_val);
        let result = self.builder.ins().uextend(types::I64, is_less);
        
        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    // Set comparison instructions (immediate variants)
    fn visit_set_lt_u_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        
        // Compare: reg1 < imm (unsigned)
        let is_less = self.builder.ins().icmp(IntCC::UnsignedLessThan, src_val, imm_val);
        let result = self.builder.ins().uextend(types::I64, is_less);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_set_lt_s_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        
        // Compare: reg1 < imm (signed)
        let is_less = self.builder.ins().icmp(IntCC::SignedLessThan, src_val, imm_val);
        let result = self.builder.ins().uextend(types::I64, is_less);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_set_gt_u_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        
        // Compare: reg1 > imm (unsigned)
        let is_greater = self.builder.ins().icmp(IntCC::UnsignedGreaterThan, src_val, imm_val);
        let result = self.builder.ins().uextend(types::I64, is_greater);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_set_gt_s_imm(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        
        // Compare: reg1 > imm (signed)
        let is_greater = self.builder.ins().icmp(IntCC::SignedGreaterThan, src_val, imm_val);
        let result = self.builder.ins().uextend(types::I64, is_greater);
        
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    // Shift immediate "alt" variants (alternative encodings) - 32-bit
    fn visit_shlo_l_imm_alt_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        // Same logic as regular shift left immediate, just different encoding format
        self.visit_shlo_l_imm_32(format)
    }

    fn visit_shlo_r_imm_alt_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        // Same logic as regular shift right immediate, just different encoding format
        self.visit_shlo_r_imm_32(format)
    }

    fn visit_shar_r_imm_alt_32(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        // Same logic as regular arithmetic shift right immediate, just different encoding format
        self.visit_shar_r_imm_32(format)
    }

    // Shift immediate "alt" variants (alternative encodings) - 64-bit
    fn visit_shlo_l_imm_alt_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        // Same logic as regular shift left immediate, just different encoding format
        self.visit_shlo_l_imm_64(format)
    }

    fn visit_shlo_r_imm_alt_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        // Same logic as regular shift right immediate, just different encoding format
        self.visit_shlo_r_imm_64(format)
    }

    fn visit_shar_r_imm_alt_64(&mut self, format: format::RRI) -> Result<(), Self::Error> {
        // Same logic as regular arithmetic shift right immediate, just different encoding format
        self.visit_shar_r_imm_64(format)
    }
}
