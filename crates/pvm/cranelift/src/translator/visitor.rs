//! Visitor implementation for PVM instructions

#![allow(unused)]

use crate::{
    constants::{context_offsets, exec_result, MAX_REGISTER_INDEX},
    Translator,
};
use core::ops::Range;
use cranelift::prelude::*;
use parser::{format, Visitor};

impl Visitor for Translator<'_> {
    type Error = anyhow::Error;

    fn visit_trap(&mut self, range: &Range<usize>) -> Result<(), Self::Error> {
        // Set trap result in context for runtime handling
        let context_ptr = self.ctx_ptr.expect("Context pointer not initialized");
        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(context_ptr, result_offset);

        // Store Trap discriminant (3)
        let trap_discriminant = self
            .builder
            .ins()
            .iconst(types::I64, exec_result::TRAP as i64);
        self.builder
            .ins()
            .store(MemFlags::new(), trap_discriminant, result_addr, 0);

        self.return_trap_with_pc(range.start)
    }

    fn visit_fallthrough(&mut self, range: &Range<usize>) -> Result<(), Self::Error> {
        let next_pc = range.end as u64;
        if let Some(&next_block) = self.blocks.get(&next_pc) {
            self.builder.ins().jump(next_block, &[]);
        } else {
            self.return_continue_with_pc(next_pc)?;
        }
        Ok(())
    }

    fn visit_load_imm(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);
        Ok(())
    }

    fn visit_load_imm_64(
        &mut self,
        format: format::REI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::REI { reg0, eimm0 } = format;
        let imm_val = self.builder.ins().iconst(types::I64, eimm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);
        Ok(())
    }

    fn visit_load_imm_jump(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, off0, imm0 } = format;
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, imm_val);

        let target_pc = (range.start as i64 + off0 as i64) as u64;
        if let Some(&target_block) = self.blocks.get(&target_pc) {
            self.builder.ins().jump(target_block, &[]);
        } else {
            // Jump to unknown target - return with jump result
            self.return_with_jump_result(target_pc)?;
        }

        Ok(())
    }

    fn visit_add_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;

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

    fn visit_add_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().iadd(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_add_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_add_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_sub_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_sub_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_mul_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_mul_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_mul_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_mul_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().imul(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_u_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
        let result_32_ext = self.builder.ins().sextend(types::I64, result_32);
        let result = self.builder.ins().select(is_zero, max_val, result_32_ext);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_div_u_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if reg0 > MAX_REGISTER_INDEX || reg1 > MAX_REGISTER_INDEX || reg2 > MAX_REGISTER_INDEX {
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

    fn visit_div_s_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_div_s_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_move_reg(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, src_val);
        Ok(())
    }

    fn visit_and(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
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

    fn visit_and_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().band(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_or(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
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

    fn visit_or_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().bor(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_xor(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
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

    fn visit_xor_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().bxor(src_val, imm_val);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_rem_u_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_rem_u_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_rem_s_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_rem_s_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shlo_l_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shlo_l_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shlo_l_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shlo_l_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shlo_r_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shlo_r_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shlo_r_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shlo_r_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shar_r_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shar_r_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shar_r_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_shar_r_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
    fn visit_leading_zero_bits_64(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        let result = self.builder.ins().clz(src_val);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_leading_zero_bits_32(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_trailing_zero_bits_64(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        let result = self.builder.ins().ctz(src_val);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_trailing_zero_bits_32(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
    fn visit_rot_l_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_rot_l_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_rot_r_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_rot_r_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
    fn visit_rot_r_64_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_rot_r_32_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
    fn visit_load_u8(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Load from linear memory
        let value = self
            .builder
            .ins()
            .load(types::I8, MemFlags::new(), final_addr, 0);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_u16(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Load from linear memory
        let value = self
            .builder
            .ins()
            .load(types::I16, MemFlags::new(), final_addr, 0);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_u32(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Load from linear memory
        let value = self
            .builder
            .ins()
            .load(types::I32, MemFlags::new(), final_addr, 0);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_u64(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Load from linear memory
        let value = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), final_addr, 0);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_i8(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit signed memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Load from linear memory with sign extension
        let unsigned_value = self
            .builder
            .ins()
            .load(types::I8, MemFlags::new(), final_addr, 0);
        let value = self.builder.ins().sextend(types::I64, unsigned_value);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_i16(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit signed memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Load from linear memory with sign extension
        let unsigned_value = self
            .builder
            .ins()
            .load(types::I16, MemFlags::new(), final_addr, 0);
        let value = self.builder.ins().sextend(types::I64, unsigned_value);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_i32(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let dst_var = self.registers[&reg0];

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit signed memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Load from linear memory with sign extension
        let unsigned_value = self
            .builder
            .ins()
            .load(types::I32, MemFlags::new(), final_addr, 0);
        let value = self.builder.ins().sextend(types::I64, unsigned_value);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    // Indirect load operations
    fn visit_load_ind_u8(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Load from linear memory
        let value = self
            .builder
            .ins()
            .load(types::I8, MemFlags::new(), final_addr, 0);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_ind_u16(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Load from linear memory
        let value = self
            .builder
            .ins()
            .load(types::I16, MemFlags::new(), final_addr, 0);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_ind_u32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Load from linear memory
        let value = self
            .builder
            .ins()
            .load(types::I32, MemFlags::new(), final_addr, 0);
        let extended = self.builder.ins().uextend(types::I64, value);

        self.builder.def_var(dst_var, extended);
        Ok(())
    }

    fn visit_load_ind_u64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Load from linear memory
        let value = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), final_addr, 0);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_ind_i8(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit signed memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Load from linear memory with sign extension
        let unsigned_value = self
            .builder
            .ins()
            .load(types::I8, MemFlags::new(), final_addr, 0);
        let value = self.builder.ins().sextend(types::I64, unsigned_value);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_ind_i16(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit signed memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Load from linear memory with sign extension
        let unsigned_value = self
            .builder
            .ins()
            .load(types::I16, MemFlags::new(), final_addr, 0);
        let value = self.builder.ins().sextend(types::I64, unsigned_value);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    fn visit_load_ind_i32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let dst_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit signed memory read
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Load from linear memory with sign extension
        let unsigned_value = self
            .builder
            .ins()
            .load(types::I32, MemFlags::new(), final_addr, 0);
        let value = self.builder.ins().sextend(types::I64, unsigned_value);

        self.builder.def_var(dst_var, value);
        Ok(())
    }

    // Branch operations - implement conditional jumps for block-based JIT
    fn visit_branch_eq(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;

        // Compare registers
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition = self.builder.ins().icmp(IntCC::Equal, reg0_val, reg1_val);

        // Calculate target addresses
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_eq_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self.builder.ins().icmp(IntCC::Equal, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ne(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition = self.builder.ins().icmp(IntCC::NotEqual, reg0_val, reg1_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ne_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self.builder.ins().icmp(IntCC::NotEqual, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_lt_u(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, reg0_val, reg1_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_lt_s(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, reg0_val, reg1_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ge_u(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThanOrEqual, reg0_val, reg1_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ge_s(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let reg0_val = self.builder.use_var(self.registers[&reg0]);
        let reg1_val = self.builder.use_var(self.registers[&reg1]);
        let condition =
            self.builder
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, reg0_val, reg1_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_lt_u_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_lt_s_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ge_u_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition =
            self.builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThanOrEqual, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ge_s_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedGreaterThanOrEqual, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    // Additional branch operations
    fn visit_branch_gt_u_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedGreaterThan, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_gt_s_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedGreaterThan, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_le_u_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThanOrEqual, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_le_s_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThanOrEqual, reg_val, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    // Jump operations
    fn visit_jump(&mut self, format: format::O, range: &Range<usize>) -> Result<(), Self::Error> {
        let format::O { off0 } = format;
        let target_pc = (range.start as i64 + format.off0 as i64) as u64;
        if let Some(&target_block) = self.blocks.get(&target_pc) {
            self.builder.ins().jump(target_block, &[]);
        } else {
            self.return_with_jump_result(target_pc)?;
        }
        Ok(())
    }

    fn visit_jump_ind(
        &mut self,
        format: format::RI,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;

        // Calculate the target address: reg0 + imm0
        let reg_val = self.builder.use_var(self.registers[&reg0]);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let target_addr = self.builder.ins().iadd(reg_val, imm_val);

        // For indirect jumps, we need to look up the address in the jump table at runtime
        // The runtime will validate and find the actual PC from the jump table
        let context_ptr = self.ctx_ptr.expect("Context pointer not initialized");
        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(context_ptr, result_offset);

        // Store the target address
        let data_offset = self.builder.ins().iconst(types::I64, 8);
        let data_addr = self.builder.ins().iadd(result_addr, data_offset);
        self.builder
            .ins()
            .store(MemFlags::new(), target_addr, data_addr, 0);

        self.djump(range.start)
    }

    // Conditional move operations
    fn visit_cmov_iz(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_cmov_iz_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_cmov_nz(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_cmov_nz_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
    fn visit_load_imm_jump_ind(
        &mut self,
        format: format::RRII,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let reg1_var = self.registers[&format.reg1];
        let reg1_val = self.builder.use_var(reg1_var);
        let imm1_val = self.builder.ins().iconst(types::I64, format.imm1 as i64);
        let target_addr = self.builder.ins().iadd(reg1_val, imm1_val);

        // THEN set the register to the new value
        let reg0_var = self.registers[&format.reg0];
        let imm0_val = self.builder.ins().iconst(types::I64, format.imm0 as i64);
        self.builder.def_var(reg0_var, imm0_val);

        // Store JumpIndirect result - let runtime handle jump table validation
        let context_ptr = self.ctx_ptr.expect("Context pointer not initialized");
        let result_offset = self
            .builder
            .ins()
            .iconst(types::I64, context_offsets::RESULT_OFFSET as i64);
        let result_addr = self.builder.ins().iadd(context_ptr, result_offset);

        // Store JumpIndirect discriminant (4) - runtime will resolve address via jump table
        let jump_discriminant = self.builder.ins().iconst(types::I64, 4); // JumpIndirect variant
        self.builder
            .ins()
            .store(MemFlags::new(), jump_discriminant, result_addr, 0);

        // Store the target address (will be resolved by runtime)
        let data_offset = self.builder.ins().iconst(types::I64, 8);
        let data_addr = self.builder.ins().iadd(result_addr, data_offset);
        self.builder
            .ins()
            .store(MemFlags::new(), target_addr, data_addr, 0);

        self.djump(range.start)
    }

    // Store operations
    fn visit_store_u8(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);
        let truncated = self.builder.ins().ireduce(types::I8, src_val);
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), truncated, final_addr, 0);

        Ok(())
    }

    fn visit_store_u16(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);
        let truncated = self.builder.ins().ireduce(types::I16, src_val);
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), truncated, final_addr, 0);

        Ok(())
    }

    fn visit_store_u32(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);
        let truncated = self.builder.ins().ireduce(types::I32, src_val);

        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), truncated, final_addr, 0);

        Ok(())
    }

    fn visit_store_u64(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src_var = self.registers[&reg0];
        let src_val = self.builder.use_var(src_var);

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), src_val, final_addr, 0);

        Ok(())
    }

    // Store immediate operations
    fn visit_store_imm_u8(
        &mut self,
        format: format::II,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I8, (imm1 as u8) as i64);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Store to linear memory with type checking
        let write_value = if self.builder.func.dfg.value_type(value).bits() > 8 {
            self.builder.ins().ireduce(types::I8, value)
        } else {
            value
        };
        self.builder
            .ins()
            .store(MemFlags::new(), write_value, final_addr, 0);

        Ok(())
    }

    fn visit_store_imm_u16(
        &mut self,
        format: format::II,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I16, (imm1 as u16) as i64);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Store to linear memory with type checking
        let write_value = if self.builder.func.dfg.value_type(value).bits() > 16 {
            self.builder.ins().ireduce(types::I16, value)
        } else {
            value
        };
        self.builder
            .ins()
            .store(MemFlags::new(), write_value, final_addr, 0);

        Ok(())
    }

    fn visit_store_imm_u32(
        &mut self,
        format: format::II,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I32, (imm1 as u32) as i64);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Store to linear memory with type checking
        let write_value = if self.builder.func.dfg.value_type(value).bits() > 32 {
            self.builder.ins().ireduce(types::I32, value)
        } else {
            value
        };
        self.builder
            .ins()
            .store(MemFlags::new(), write_value, final_addr, 0);

        Ok(())
    }

    fn visit_store_imm_u64(
        &mut self,
        format: format::II,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;

        // Create address from immediate
        let address = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I64, imm1 as i64);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, address);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), value, final_addr, 0);

        Ok(())
    }

    // Indirect store operations
    fn visit_store_ind_u8(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), truncated, final_addr, 0);

        Ok(())
    }

    fn visit_store_ind_u16(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), truncated, final_addr, 0);

        Ok(())
    }

    fn visit_store_ind_u32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), truncated, final_addr, 0);

        Ok(())
    }

    fn visit_store_ind_u64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg0];
        let addr_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), src_val, final_addr, 0);

        Ok(())
    }

    // Store immediate indirect operations
    fn visit_store_imm_ind_u8(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I8, (imm1 as u8) as i64);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Store to linear memory with type checking
        let write_value = if self.builder.func.dfg.value_type(value).bits() > 8 {
            self.builder.ins().ireduce(types::I8, value)
        } else {
            value
        };
        self.builder
            .ins()
            .store(MemFlags::new(), write_value, final_addr, 0);

        Ok(())
    }

    fn visit_store_imm_ind_u16(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Create immediate value
        let value = self.builder.ins().iconst(types::I16, (imm1 as u16) as i64);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Store to linear memory with type checking
        let write_value = if self.builder.func.dfg.value_type(value).bits() > 16 {
            self.builder.ins().ireduce(types::I16, value)
        } else {
            value
        };
        self.builder
            .ins()
            .store(MemFlags::new(), write_value, final_addr, 0);

        Ok(())
    }

    fn visit_store_imm_ind_u32(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Check store boundaries to detect cross-page writes (4 bytes for u32)
        self.check_store_boundaries(effective_addr, 4)?;

        // Create immediate value
        let value = self.builder.ins().iconst(types::I32, (imm1 as u32) as i64);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Store to linear memory with type checking
        let write_value = if self.builder.func.dfg.value_type(value).bits() > 32 {
            self.builder.ins().ireduce(types::I32, value)
        } else {
            value
        };
        self.builder
            .ins()
            .store(MemFlags::new(), write_value, final_addr, 0);

        Ok(())
    }

    fn visit_store_imm_ind_u64(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr_var = self.registers[&reg0];
        let addr_val = self.builder.use_var(addr_var);

        // Calculate effective address with offset
        let offset = self.builder.ins().iconst(types::I64, imm0 as i64);
        let effective_addr = self.builder.ins().iadd(addr_val, offset);

        // Check store boundaries to detect cross-page writes (8 bytes for u64)
        self.check_store_boundaries(effective_addr, 8)?;

        // Create immediate value
        let value = self.builder.ins().iconst(types::I64, imm1 as i64);

        // Emit memory write
        // Get linear memory base and calculate final address
        let memory_base = self.get_memory_base();
        let final_addr = self.builder.ins().iadd(memory_base, effective_addr);

        // Store to linear memory
        self.builder
            .ins()
            .store(MemFlags::new(), value, final_addr, 0);

        Ok(())
    }

    // === MISSING INSTRUCTION IMPLEMENTATIONS ===

    // Negate and add immediate instructions
    fn visit_neg_add_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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

    fn visit_neg_add_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
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
    fn visit_set_lt_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // Compare: reg0 < reg1 (unsigned)
        let is_less = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, src0_val, src1_val);
        let result = self.builder.ins().uextend(types::I64, is_less);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_set_lt_s(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // Compare: reg0 < reg1 (signed)
        let is_less = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, src0_val, src1_val);
        let result = self.builder.ins().uextend(types::I64, is_less);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    // Set comparison instructions (immediate variants)
    fn visit_set_lt_u_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Compare: reg1 < imm (unsigned)
        let is_less = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, src_val, imm_val);
        let result = self.builder.ins().uextend(types::I64, is_less);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_set_lt_s_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Compare: reg1 < imm (signed)
        let is_less = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, src_val, imm_val);
        let result = self.builder.ins().uextend(types::I64, is_less);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_set_gt_u_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Compare: reg1 > imm (unsigned)
        let is_greater = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedGreaterThan, src_val, imm_val);
        let result = self.builder.ins().uextend(types::I64, is_greater);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_set_gt_s_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Compare: reg1 > imm (signed)
        let is_greater = self
            .builder
            .ins()
            .icmp(IntCC::SignedGreaterThan, src_val, imm_val);
        let result = self.builder.ins().uextend(types::I64, is_greater);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    // Shift immediate "alt" variants (alternative encodings) - 32-bit
    fn visit_shlo_l_imm_alt_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        // Alt version: shift imm0 by src_reg (roles reversed from regular version)
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift_var = self.registers[&reg1];
        let shift_val = self.builder.use_var(shift_var);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);

        // Mask shift amount to avoid undefined behavior
        let safe_shift = self.builder.ins().band_imm(shift_32, 31);

        // Shift immediate value by register content (left shift)
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().ishl(imm_val, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shlo_r_imm_alt_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        // Alt version: shift imm0 by src_reg (roles reversed from regular version)
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift_var = self.registers[&reg1];
        let shift_val = self.builder.use_var(shift_var);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);

        // Mask shift amount to avoid undefined behavior
        let safe_shift = self.builder.ins().band_imm(shift_32, 31);

        // Shift immediate value by register content
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().ushr(imm_val, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_shar_r_imm_alt_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        // Alt version: arithmetic shift imm0 by src_reg (roles reversed from regular version)
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift_var = self.registers[&reg1];
        let shift_val = self.builder.use_var(shift_var);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);

        // Mask shift amount to avoid undefined behavior
        let safe_shift = self.builder.ins().band_imm(shift_32, 31);

        // Arithmetic shift immediate value by register content
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().sshr(imm_val, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    // Shift immediate "alt" variants (alternative encodings) - 64-bit
    fn visit_shlo_l_imm_alt_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        // Alt version: shift imm0 by src_reg (roles reversed from regular version)
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift_var = self.registers[&reg1];
        let shift_val = self.builder.use_var(shift_var);

        // Mask shift amount to avoid undefined behavior
        let safe_shift = self.builder.ins().band_imm(shift_val, 63);

        // Shift immediate value by register content (left shift)
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().ishl(imm_val, safe_shift);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shlo_r_imm_alt_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        // Alt version: shift imm0 by src_reg (roles reversed from regular version)
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift_var = self.registers[&reg1];
        let shift_val = self.builder.use_var(shift_var);

        // Mask shift amount to avoid undefined behavior
        let safe_shift = self.builder.ins().band_imm(shift_val, 63);

        // Shift immediate value by register content
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().ushr(imm_val, safe_shift);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_shar_r_imm_alt_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        // Alt version: arithmetic shift imm0 by src_reg (roles reversed from regular version)
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift_var = self.registers[&reg1];
        let shift_val = self.builder.use_var(shift_var);

        // Mask shift amount to avoid undefined behavior
        let safe_shift = self.builder.ins().band_imm(shift_val, 63);

        // Arithmetic shift immediate value by register content
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().sshr(imm_val, safe_shift);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    // Missing visitor methods for RISC-V tests

    fn visit_mul_upper_s_s(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // For upper bits of signed*signed multiplication
        // We need to handle this as 64-bit * 64-bit = 128-bit and get upper 64 bits
        let result = self.builder.ins().smulhi(src0_val, src1_val);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_mul_upper_u_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // For upper bits of unsigned*unsigned multiplication
        let result = self.builder.ins().umulhi(src0_val, src1_val);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_mul_upper_s_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // For upper bits of signed*unsigned multiplication (mulhsu)
        // First operand is signed, second operand is unsigned
        // We can emulate this using: mulhu(a,b) - (a < 0 ? b : 0)
        // This is the standard way to convert unsigned high multiply to signed×unsigned

        let unsigned_high = self.builder.ins().umulhi(src0_val, src1_val);

        // Check if first operand is negative (signed)
        let zero = self.builder.ins().iconst(types::I64, 0);
        let src0_negative = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, src0_val, zero);

        // If src0 is negative, subtract src1 from the unsigned high result
        let correction = self.builder.ins().select(src0_negative, src1_val, zero);
        let result = self.builder.ins().isub(unsigned_high, correction);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_count_set_bits_64(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Count set bits (population count)
        let result = self.builder.ins().popcnt(src_val);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_count_set_bits_32(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Truncate to 32-bit, count set bits, then extend back
        let src32 = self.builder.ins().ireduce(types::I32, src_val);
        let count32 = self.builder.ins().popcnt(src32);
        let result = self.builder.ins().uextend(types::I64, count32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_and_inv(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // AND with inverted: result = src0 & (~src1)
        let inv_src1 = self.builder.ins().bnot(src1_val);
        let result = self.builder.ins().band(src0_val, inv_src1);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_or_inv(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // OR with inverted: result = src0 | (~src1)
        let inv_src1 = self.builder.ins().bnot(src1_val);
        let result = self.builder.ins().bor(src0_val, inv_src1);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_xnor(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // XNOR: result = ~(src0 ^ src1) = src0 ^ ~src1
        let xor_result = self.builder.ins().bxor(src0_val, src1_val);
        let result = self.builder.ins().bnot(xor_result);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_max(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // Signed maximum
        let cmp = self
            .builder
            .ins()
            .icmp(IntCC::SignedGreaterThan, src0_val, src1_val);
        let result = self.builder.ins().select(cmp, src0_val, src1_val);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_max_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // Unsigned maximum
        let cmp = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedGreaterThan, src0_val, src1_val);
        let result = self.builder.ins().select(cmp, src0_val, src1_val);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_min(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // Signed minimum
        let cmp = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, src0_val, src1_val);
        let result = self.builder.ins().select(cmp, src0_val, src1_val);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_min_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_var = self.registers[&reg0];
        let src1_var = self.registers[&reg1];
        let src0_val = self.builder.use_var(src0_var);
        let src1_val = self.builder.use_var(src1_var);

        // Unsigned minimum
        let cmp = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, src0_val, src1_val);
        let result = self.builder.ins().select(cmp, src0_val, src1_val);

        let dst_var = self.registers[&reg2];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_reverse_bytes(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Byte reversal (endianness swap)
        let result = self.builder.ins().bswap(src_val);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_sign_extend_8(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Sign extend from 8 bits
        let src8 = self.builder.ins().ireduce(types::I8, src_val);
        let result = self.builder.ins().sextend(types::I64, src8);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_sign_extend_16(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Sign extend from 16 bits
        let src16 = self.builder.ins().ireduce(types::I16, src_val);
        let result = self.builder.ins().sextend(types::I64, src16);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_zero_extend_16(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_var = self.registers[&reg1];
        let src_val = self.builder.use_var(src_var);

        // Zero extend from 16 bits (mask to 16 bits)
        let result = self.builder.ins().band_imm(src_val, 0xFFFF);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }

    fn visit_ecalli(
        &mut self,
        format: format::I,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::I { imm0 } = format;
        // ecalli triggers a host call - we need to return an error that will be caught
        // by the runtime to handle the host call
        // For now, we'll just return an error to indicate host call needed
        Err(anyhow::anyhow!("Host call requested: {}", imm0))
    }

    fn visit_sbrk(&mut self, format: format::RR, _range: &Range<usize>) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;

        // sbrk adjusts the heap pointer and returns the old value
        // We need to get the current heap pointer from memory context
        // For now, we'll implement a basic version that returns 0
        // This will need to be updated to interact with the memory management system

        // Get the increment value from reg1
        let _increment_var = self.registers[&reg1];
        let _increment_val = self.builder.use_var(_increment_var);

        // TODO: Implement proper heap pointer management
        // For now, return 0 in reg0 to indicate heap start
        let zero = self.builder.ins().iconst(types::I64, 0);
        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, zero);

        Ok(())
    }

    fn visit_rot_r_32_imm_alt(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;

        // Get rotation amount from reg1
        let shift_var = self.registers[&reg1];
        let shift_val = self.builder.use_var(shift_var);

        // Convert immediate to 32-bit and create value
        let imm_val = self.builder.ins().iconst(types::I32, (imm0 as u32) as i64);

        // Mask rotation amount to 0-31
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_32, mask);

        // Perform rotation
        let result_32 = self.builder.ins().rotr(imm_val, safe_shift);

        // Sign extend to 64 bits
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result_64);
        Ok(())
    }

    fn visit_rot_r_64_imm_alt(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;

        // Get rotation amount from reg1
        let shift_var = self.registers[&reg1];
        let shift_val = self.builder.use_var(shift_var);

        // Create immediate value
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);

        // Mask rotation amount to 0-63
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(shift_val, mask);

        // Perform rotation
        let result = self.builder.ins().rotr(imm_val, safe_shift);

        let dst_var = self.registers[&reg0];
        self.builder.def_var(dst_var, result);
        Ok(())
    }
}
