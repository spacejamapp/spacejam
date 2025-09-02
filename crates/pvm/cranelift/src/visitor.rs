//! Visitor implementation for PVM instructions

use crate::Exit;
use crate::Translator;
use core::ops::Range;
use cranelift::prelude::*;
use parser::{format, Visitor};

impl Visitor for Translator<'_> {
    type Error = anyhow::Error;

    fn visit_add_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let result = self.builder.ins().iadd(src0_val, src1_val);
        let result_32 = self.builder.ins().ireduce(types::I32, result);
        let result_sext = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg2, result_sext);
        Ok(())
    }

    fn visit_add_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let result = self.builder.ins().iadd(src0_val, src1_val);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_add_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let result = self.builder.ins().iadd_imm(src_val, imm0 as i64);
        let result_32 = self.builder.ins().ireduce(types::I32, result);
        let result_sext = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_sext);
        Ok(())
    }

    fn visit_add_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let result = self.builder.ins().iadd_imm(src_val, imm0 as i64);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_and(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let result = self.builder.ins().band(src0_val, src1_val);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_and_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let result = self.builder.ins().band_imm(src_val, imm0 as i64);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_and_inv(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let inv_src1 = self.builder.ins().bnot(src1);
        let result = self.builder.ins().band(src0, inv_src1);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_branch_eq(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let lhs = self.rget(reg0);
        let rhs = self.rget(reg1);
        let condition = self.builder.ins().icmp(IntCC::Equal, lhs, rhs);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_eq_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition = self.builder.ins().icmp_imm(IntCC::Equal, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ge_s(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let lhs = self.rget(reg0);
        let rhs = self.rget(reg1);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ge_s_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition =
            self.builder
                .ins()
                .icmp_imm(IntCC::SignedGreaterThanOrEqual, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ge_u(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let lhs = self.rget(reg0);
        let rhs = self.rget(reg1);
        let condition = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedGreaterThanOrEqual, lhs, rhs);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ge_u_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition =
            self.builder
                .ins()
                .icmp_imm(IntCC::UnsignedGreaterThanOrEqual, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_gt_s_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition = self
            .builder
            .ins()
            .icmp_imm(IntCC::SignedGreaterThan, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }
    fn visit_branch_gt_u_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedGreaterThan, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_le_s_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition = self
            .builder
            .ins()
            .icmp_imm(IntCC::SignedLessThanOrEqual, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_le_u_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition =
            self.builder
                .ins()
                .icmp_imm(IntCC::UnsignedLessThanOrEqual, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_lt_s(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let lhs = self.rget(reg0);
        let rhs = self.rget(reg1);
        let condition = self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_lt_s_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition = self
            .builder
            .ins()
            .icmp_imm(IntCC::SignedLessThan, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_lt_u(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let lhs = self.rget(reg0);
        let rhs = self.rget(reg1);
        let condition = self.builder.ins().icmp(IntCC::UnsignedLessThan, lhs, rhs);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_lt_u_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThan, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ne(
        &mut self,
        format: format::RRO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRO { reg0, reg1, off0 } = format;
        let lhs = self.rget(reg0);
        let rhs = self.rget(reg1);
        let condition = self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }

    fn visit_branch_ne_imm(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, imm0, off0 } = format;
        let lhs = self.rget(reg0);
        let condition = self
            .builder
            .ins()
            .icmp_imm(IntCC::NotEqual, lhs, imm0 as i64);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        self.branch(condition, target_pc, range.end as u64)
    }
    fn visit_cmov_iz(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src = self.rget(reg0);
        let cond = self.rget(reg1);
        let dst = self.rget(reg2);
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, cond, zero);

        // Select between reg0 (if condition met) or current reg2 value (if condition not met)
        let new_val = self.builder.ins().select(is_zero, src, dst);
        self.rset(reg2, new_val);
        Ok(())
    }

    fn visit_cmov_iz_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg0);
        let cond = self.rget(reg1);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let zero = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, cond, zero);

        // Select between immediate (if zero) or current reg0 value (if not zero)
        let new_val = self.builder.ins().select(is_zero, imm_val, src);
        self.rset(reg0, new_val);
        Ok(())
    }

    fn visit_cmov_nz(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src = self.rget(reg0);
        let cond = self.rget(reg1);
        let dst = self.rget(reg2);
        let zero = self.builder.ins().iconst(types::I64, 0);
        let not_zero = self.builder.ins().icmp(IntCC::NotEqual, cond, zero);
        let new_val = self.builder.ins().select(not_zero, src, dst);
        self.rset(reg2, new_val);
        Ok(())
    }

    fn visit_cmov_nz_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg0);
        let cond = self.rget(reg1);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let zero = self.builder.ins().iconst(types::I64, 0);
        let not_zero = self.builder.ins().icmp(IntCC::NotEqual, cond, zero);
        let new_val = self.builder.ins().select(not_zero, imm_val, src);
        self.rset(reg0, new_val);
        Ok(())
    }

    fn visit_count_set_bits_32(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src = self.rget(reg1);
        let src32 = self.builder.ins().ireduce(types::I32, src);
        let count32 = self.builder.ins().popcnt(src32);
        let result = self.builder.ins().uextend(types::I64, count32);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_count_set_bits_64(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src = self.rget(reg1);
        let result = self.builder.ins().popcnt(src);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_div_s_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);
        let result = self.safe_div_s32(dividend, divisor);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_div_s_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);
        let result = self.safe_div_s64(dividend, divisor);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_div_u_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);
        let result = self.safe_div_u32(dividend, divisor);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_div_u_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);
        let result = self.safe_div_u64(dividend, divisor);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_ecalli(
        &mut self,
        format: format::I,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::I { imm0 } = format;
        let index = self.builder.ins().iconst(types::I32, imm0 as i64);
        let inst = self
            .builder
            .ins()
            .call(self.host["call"], &[index, self.pool.ctx]);
        let result = self.builder.inst_results(inst)[0];
        let panic = self.builder.ins().iconst(types::I8, 1);
        let is_panic = self.builder.ins().icmp(IntCC::Equal, result, panic);

        // Return with panic if result equals panic
        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        self.builder
            .ins()
            .brif(is_panic, then_block, &[], else_block, &[]);
        self.builder.switch_to_block(then_block);
        self.return_(Exit::HostCallPanicked);
        self.builder.switch_to_block(else_block);
        Ok(())
    }

    fn visit_fallthrough(&mut self, range: &Range<usize>) -> Result<(), Self::Error> {
        if let Some(block) = self.blocks.get(&(range.end as u64)) {
            self.builder.ins().jump(*block, &[]);
        } else {
            self.burn_gas(self.pool.one);
            self.return_(Exit::ProgramNotTerminated);
        }

        Ok(())
    }
    fn visit_jump(&mut self, format: format::O, range: &Range<usize>) -> Result<(), Self::Error> {
        let format::O { off0 } = format;
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        let target_block = self.blocks[&target_pc];
        self.builder.ins().jump(target_block, &[]);
        Ok(())
    }

    fn visit_jump_ind(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let lhs = self.rget(reg0);
        let target = self.builder.ins().iadd_imm(lhs, imm0 as i64);
        self.djump(target)
    }

    fn visit_leading_zero_bits_32(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_val = self.rget(reg1);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let result_32 = self.builder.ins().clz(src_32);
        let result_64 = self.builder.ins().uextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_leading_zero_bits_64(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_val = self.rget(reg1);
        let result = self.builder.ins().clz(src_val);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_load_i16(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let unsigned_value = self.mget_imm(imm0 as i64, types::I16);
        let value = self.builder.ins().sextend(types::I64, unsigned_value);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_load_i32(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let unsigned_value = self.mget_imm(imm0 as i64, types::I32);
        let value = self.builder.ins().sextend(types::I64, unsigned_value);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_load_i8(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let unsigned_value = self.mget_imm(imm0 as i64, types::I8);
        let value = self.builder.ins().sextend(types::I64, unsigned_value);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_load_imm(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        self.rset(reg0, imm_val);
        Ok(())
    }

    fn visit_load_imm_64(
        &mut self,
        format: format::REI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::REI { reg0, eimm0 } = format;
        let imm_val = self.builder.ins().iconst(types::I64, eimm0 as i64);
        self.rset(reg0, imm_val);
        Ok(())
    }

    fn visit_load_imm_jump(
        &mut self,
        format: format::RIO,
        range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RIO { reg0, off0, imm0 } = format;
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        self.rset(reg0, imm_val);
        let target_pc = (range.start as i64 + off0 as i64) as u64;
        let target_block = self.blocks[&target_pc];
        self.builder.ins().jump(target_block, &[]);
        Ok(())
    }

    fn visit_load_imm_jump_ind(
        &mut self,
        format: format::RRII,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRII {
            reg0,
            reg1,
            imm0,
            imm1,
        } = format;
        let src = self.rget(reg1);
        let target = self.builder.ins().iadd_imm(src, imm1 as i64);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        self.rset(reg0, imm_val);
        self.djump(target)
    }

    fn visit_load_ind_i16(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let unsigned = self.mget(addr, imm0 as i64, types::I16);
        let value = self.builder.ins().sextend(types::I64, unsigned);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_load_ind_i32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let unsigned = self.mget(addr, imm0 as i64, types::I32);
        let value = self.builder.ins().sextend(types::I64, unsigned);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_load_ind_i8(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let unsigned = self.mget(addr, imm0 as i64, types::I8);
        let value = self.builder.ins().sextend(types::I64, unsigned);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_load_ind_u16(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value = self.mget(addr, imm0 as i64, types::I16);
        let extended = self.builder.ins().uextend(types::I64, value);
        self.rset(reg0, extended);
        Ok(())
    }

    fn visit_load_ind_u32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value = self.mget(addr, imm0 as i64, types::I32);
        let extended = self.builder.ins().uextend(types::I64, value);
        self.rset(reg0, extended);
        Ok(())
    }

    fn visit_load_ind_u64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value = self.mget(addr, imm0 as i64, types::I64);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_load_ind_u8(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value = self.mget(addr, imm0 as i64, types::I8);
        let extended = self.builder.ins().uextend(types::I64, value);
        self.rset(reg0, extended);
        Ok(())
    }

    fn visit_load_u16(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let value = self.mget_imm(imm0 as i64, types::I16);
        let extended = self.builder.ins().uextend(types::I64, value);
        self.rset(reg0, extended);
        Ok(())
    }

    fn visit_load_u32(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let value = self.mget_imm(imm0 as i64, types::I32);
        let extended = self.builder.ins().uextend(types::I64, value);
        self.rset(reg0, extended);
        Ok(())
    }

    fn visit_load_u64(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let value = self.mget_imm(imm0 as i64, types::I64);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_load_u8(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let value = self.mget_imm(imm0 as i64, types::I8);
        let extended = self.builder.ins().uextend(types::I64, value);
        self.rset(reg0, extended);
        Ok(())
    }

    fn visit_max(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let cmp = self
            .builder
            .ins()
            .icmp(IntCC::SignedGreaterThan, src0, src1);
        let result = self.builder.ins().select(cmp, src0, src1);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_max_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let cmp = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedGreaterThan, src0, src1);
        let result = self.builder.ins().select(cmp, src0, src1);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_min(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let cmp = self.builder.ins().icmp(IntCC::SignedLessThan, src0, src1);
        let result = self.builder.ins().select(cmp, src0, src1);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_min_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let src0_val = src0;
        let src1_val = src1;
        let cmp = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, src0_val, src1_val);
        let result = self.builder.ins().select(cmp, src0_val, src1_val);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_move_reg(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_val = self.rget(reg1);
        self.rset(reg0, src_val);
        Ok(())
    }

    fn visit_mul_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let result = self.builder.ins().imul(src0_val, src1_val);
        let result_32 = self.builder.ins().ireduce(types::I32, result);
        let result_sext = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg2, result_sext);
        Ok(())
    }

    fn visit_mul_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let result = self.builder.ins().imul(src0_val, src1_val);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_mul_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let result = self.builder.ins().imul_imm(src_val, imm0 as i64);
        let result_32 = self.builder.ins().ireduce(types::I32, result);
        let result_sext = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_sext);
        Ok(())
    }

    fn visit_mul_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let result = self.builder.ins().imul_imm(src_val, imm0 as i64);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_mul_upper_s_s(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let src0_val = src0;
        let src1_val = src1;
        let result = self.builder.ins().smulhi(src0_val, src1_val);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_mul_upper_s_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let src0_val = src0;
        let src1_val = src1;
        let unsigned_high = self.builder.ins().umulhi(src0_val, src1_val);
        let zero = self.builder.ins().iconst(types::I64, 0);
        let src0_negative = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, src0_val, zero);
        let correction = self.builder.ins().select(src0_negative, src1_val, zero);
        let result = self.builder.ins().isub(unsigned_high, correction);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_mul_upper_u_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let src0_val = src0;
        let src1_val = src1;
        let result = self.builder.ins().umulhi(src0_val, src1_val);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_neg_add_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg1);
        let src_32 = self.builder.ins().ireduce(types::I32, src);
        let negated = self.builder.ins().ineg(src_32);
        let result_32 = self.builder.ins().iadd_imm(negated, imm0 as i64);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_neg_add_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg1);
        let src_64 = src;
        let negated = self.builder.ins().ineg(src_64);
        let result_64 = self.builder.ins().iadd_imm(negated, imm0 as i64);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_or(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let result = self.builder.ins().bor(src0_val, src1_val);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_or_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let result = self.builder.ins().bor_imm(src_val, imm0 as i64);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_or_inv(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let src0_val = src0;
        let src1_val = src1;
        let inv_src1 = self.builder.ins().bnot(src1_val);
        let result = self.builder.ins().bor(src0_val, inv_src1);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_rem_s_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);
        let result = self.safe_rem_s32(dividend, divisor);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_rem_s_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);
        let result = self.safe_rem_s64(dividend, divisor);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_rem_u_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);
        let result = self.safe_rem_u32(dividend, divisor);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_rem_u_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);
        let result = self.safe_rem_u64(dividend, divisor);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_reverse_bytes(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src = self.rget(reg1);
        let result = self.builder.ins().bswap(src);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_rot_l_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_val = self.rget(reg0);
        let shift_val = self.rget(reg1);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_32, mask);
        let result_32 = self.builder.ins().rotl(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg2, result_64);
        Ok(())
    }

    fn visit_rot_l_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_val = self.rget(reg0);
        let shift_val = self.rget(reg1);
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result = self.builder.ins().rotl(src_val, safe_shift);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_rot_r_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_val = self.rget(reg0);
        let shift_val = self.rget(reg1);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift_val);
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_32, mask);
        let result_32 = self.builder.ins().rotr(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg2, result_64);
        Ok(())
    }

    fn visit_rot_r_32_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().rotr_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_rot_r_32_imm_alt(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1);
        let imm_val = self.builder.ins().iconst(types::I32, (imm0 as u32) as i64);
        let shift_32 = self.builder.ins().ireduce(types::I32, shift);
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_32, mask);
        let result_32 = self.builder.ins().rotr(imm_val, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_rot_r_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src_val = self.rget(reg0);
        let shift_val = self.rget(reg1);
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result = self.builder.ins().rotr(src_val, safe_shift);
        self.rset(reg2, result);
        Ok(())
    }
    fn visit_rot_r_64_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().rotr_imm(src_val, safe_shift);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_rot_r_64_imm_alt(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1);
        let shift_val = shift;
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result = self.builder.ins().rotr(imm_val, safe_shift);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_sbrk(&mut self, format: format::RR, _range: &Range<usize>) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let target = self.builder.ins().iconst(types::I8, reg0 as i64);
        let increment = self.builder.ins().iconst(types::I8, reg1 as i64);
        let _inst = self
            .builder
            .ins()
            .call(self.host["sbrk"], &[self.pool.ctx, target, increment]);
        Ok(())
    }

    fn visit_set_gt_s_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg1);
        let is_greater = self
            .builder
            .ins()
            .icmp_imm(IntCC::SignedGreaterThan, src, imm0 as i64);
        let result = self.builder.ins().uextend(types::I64, is_greater);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_set_gt_u_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg1);
        let is_greater = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedGreaterThan, src, imm0 as i64);
        let result = self.builder.ins().uextend(types::I64, is_greater);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_set_lt_s(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let src0_val = src0;
        let src1_val = src1;
        let is_less = self
            .builder
            .ins()
            .icmp(IntCC::SignedLessThan, src0_val, src1_val);
        let result = self.builder.ins().uextend(types::I64, is_less);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_set_lt_s_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg1);
        let is_less = self
            .builder
            .ins()
            .icmp_imm(IntCC::SignedLessThan, src, imm0 as i64);
        let result = self.builder.ins().uextend(types::I64, is_less);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_set_lt_u(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let src0_val = src0;
        let src1_val = src1;
        let is_less = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, src0_val, src1_val);
        let result = self.builder.ins().uextend(types::I64, is_less);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_set_lt_u_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg1);
        let is_less = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThan, src, imm0 as i64);
        let result = self.builder.ins().uextend(types::I64, is_less);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_shar_r_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let shift_val = self.builder.ins().ireduce(types::I32, src1_val);
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result_32 = self.builder.ins().sshr(src0_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg2, result_64);
        Ok(())
    }

    fn visit_shar_r_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let lhs = self.rget(reg0);
        let rhs = self.rget(reg1);
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(rhs, mask);
        let result = self.builder.ins().sshr(lhs, safe_shift);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_shar_r_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().sshr_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_shar_r_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().sshr_imm(src_val, safe_shift);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_shar_r_imm_alt_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1);
        let shift_val = self.builder.ins().ireduce(types::I32, shift);
        let safe_shift = self.builder.ins().band_imm(shift_val, 31);
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().sshr(imm_val, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_shar_r_imm_alt_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1);
        let shift_val = shift;
        let safe_shift = self.builder.ins().band_imm(shift_val, 63);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().sshr(imm_val, safe_shift);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_shlo_l_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let shift_val = self.builder.ins().ireduce(types::I32, src1_val);
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result_32 = self.builder.ins().ishl(src0_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg2, result_64);
        Ok(())
    }

    fn visit_shlo_l_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(src1_val, mask);
        let result = self.builder.ins().ishl(src0_val, safe_shift);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_shlo_l_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().ishl_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_shlo_l_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().ishl_imm(src_val, safe_shift);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_shlo_l_imm_alt_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1);
        let shift_val = self.builder.ins().ireduce(types::I32, shift);
        let safe_shift = self.builder.ins().band_imm(shift_val, 31);
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().ishl(imm_val, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_shlo_l_imm_alt_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1);
        let shift_val = shift;
        let safe_shift = self.builder.ins().band_imm(shift_val, 63);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().ishl(imm_val, safe_shift);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_shlo_r_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let src0_32 = self.builder.ins().ireduce(types::I32, src0_val);
        let shift_val = self.builder.ins().ireduce(types::I32, src1_val);
        let mask = self.builder.ins().iconst(types::I32, 31);
        let safe_shift = self.builder.ins().band(shift_val, mask);
        let result_32 = self.builder.ins().ushr(src0_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg2, result_64);
        Ok(())
    }

    fn visit_shlo_r_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let mask = self.builder.ins().iconst(types::I64, 63);
        let safe_shift = self.builder.ins().band(src1_val, mask);
        let result = self.builder.ins().ushr(src0_val, safe_shift);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_shlo_r_imm_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let safe_shift = (imm0 % 32) as i64;
        let result_32 = self.builder.ins().ushr_imm(src_32, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_shlo_r_imm_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let safe_shift = (imm0 % 64) as i64;
        let result = self.builder.ins().ushr_imm(src_val, safe_shift);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_shlo_r_imm_alt_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1);
        let shift_val = self.builder.ins().ireduce(types::I32, shift);
        let safe_shift = self.builder.ins().band_imm(shift_val, 31);
        let imm_val = self.builder.ins().iconst(types::I32, imm0 as i64);
        let result_32 = self.builder.ins().ushr(imm_val, safe_shift);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_shlo_r_imm_alt_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1);
        let shift_val = shift;
        let safe_shift = self.builder.ins().band_imm(shift_val, 63);
        let imm_val = self.builder.ins().iconst(types::I64, imm0 as i64);
        let result = self.builder.ins().ushr(imm_val, safe_shift);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_sign_extend_16(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src = self.rget(reg1);
        let src16 = self.builder.ins().ireduce(types::I16, src);
        let result = self.builder.ins().sextend(types::I64, src16);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_sign_extend_8(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src = self.rget(reg1);
        let src8 = self.builder.ins().ireduce(types::I8, src);
        let result = self.builder.ins().sextend(types::I64, src8);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_store_imm_ind_u16(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr = self.rget(reg0);
        let value = self.builder.ins().iconst(types::I16, (imm1 as u16) as i64);
        let write_value = if self.builder.func.dfg.value_type(value).bits() > 16 {
            self.builder.ins().ireduce(types::I16, value)
        } else {
            value
        };
        self.mset(addr, imm0 as i64, write_value);
        Ok(())
    }

    fn visit_store_imm_ind_u32(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr = self.rget(reg0);
        let value = self.builder.ins().iconst(types::I32, (imm1 as u32) as i64);
        let write_value = if self.builder.func.dfg.value_type(value).bits() > 32 {
            self.builder.ins().ireduce(types::I32, value)
        } else {
            value
        };
        self.mset(addr, imm0 as i64, write_value);
        Ok(())
    }

    fn visit_store_imm_ind_u64(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr = self.rget(reg0);
        let value = self.builder.ins().iconst(types::I64, imm1 as i64);
        self.mset(addr, imm0 as i64, value);
        Ok(())
    }
    fn visit_store_imm_ind_u8(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RII { reg0, imm0, imm1 } = format;
        let addr = self.rget(reg0);
        let value = self.builder.ins().iconst(types::I8, (imm1 as u8) as i64);
        let write_value = if self.builder.func.dfg.value_type(value).bits() > 8 {
            self.builder.ins().ireduce(types::I8, value)
        } else {
            value
        };
        self.mset(addr, imm0 as i64, write_value);
        Ok(())
    }

    fn visit_store_imm_u16(
        &mut self,
        format: format::II,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;
        self.mset_iimm(imm0 as i64, (imm1 as u16) as i64, types::I16);
        Ok(())
    }

    fn visit_store_imm_u32(
        &mut self,
        format: format::II,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;
        self.mset_iimm(imm0 as i64, (imm1 as u32) as i64, types::I32);
        Ok(())
    }

    fn visit_store_imm_u64(
        &mut self,
        format: format::II,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;
        self.mset_iimm(imm0 as i64, imm1 as i64, types::I64);
        Ok(())
    }

    fn visit_store_imm_u8(
        &mut self,
        format: format::II,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::II { imm0, imm1 } = format;
        self.mset_iimm(imm0 as i64, (imm1 as u8) as i64, types::I8);
        Ok(())
    }

    fn visit_store_ind_u16(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg0);
        let addr = self.rget(reg1);
        let truncated = self.builder.ins().ireduce(types::I16, src);
        self.mset(addr, imm0 as i64, truncated);
        Ok(())
    }

    fn visit_store_ind_u32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg0);
        let addr = self.rget(reg1);
        let truncated = self.builder.ins().ireduce(types::I32, src);
        self.mset(addr, imm0 as i64, truncated);
        Ok(())
    }

    fn visit_store_ind_u64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg0);
        let addr = self.rget(reg1);
        self.mset(addr, imm0 as i64, src);
        Ok(())
    }
    fn visit_store_ind_u8(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src = self.rget(reg0);
        let addr = self.rget(reg1);
        let truncated = self.builder.ins().ireduce(types::I8, src);
        self.mset(addr, imm0 as i64, truncated);
        Ok(())
    }

    fn visit_store_u16(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src = self.rget(reg0);
        let truncated = self.builder.ins().ireduce(types::I16, src);
        self.mset_imm(imm0 as i64, truncated);
        Ok(())
    }

    fn visit_store_u32(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src = self.rget(reg0);
        let truncated = self.builder.ins().ireduce(types::I32, src);
        self.mset_imm(imm0 as i64, truncated);
        Ok(())
    }

    fn visit_store_u64(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src = self.rget(reg0);
        self.mset_imm(imm0 as i64, src);
        Ok(())
    }
    fn visit_store_u8(
        &mut self,
        format: format::RI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RI { reg0, imm0 } = format;
        let src = self.rget(reg0);
        let truncated = self.builder.ins().ireduce(types::I8, src);
        self.mset_imm(imm0 as i64, truncated);
        Ok(())
    }

    fn visit_sub_32(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let result = self.builder.ins().isub(src0_val, src1_val);
        let result_32 = self.builder.ins().ireduce(types::I32, result);
        let result_sext = self.builder.ins().sextend(types::I64, result_32);
        self.rset(reg2, result_sext);
        Ok(())
    }

    fn visit_sub_64(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let result = self.builder.ins().isub(src0_val, src1_val);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_trailing_zero_bits_32(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_val = self.rget(reg1);
        let src_32 = self.builder.ins().ireduce(types::I32, src_val);
        let result_32 = self.builder.ins().ctz(src_32);
        let result_64 = self.builder.ins().uextend(types::I64, result_32);
        self.rset(reg0, result_64);
        Ok(())
    }

    fn visit_trailing_zero_bits_64(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src_val = self.rget(reg1);
        let result = self.builder.ins().ctz(src_val);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_trap(&mut self, _range: &Range<usize>) -> Result<(), Self::Error> {
        self.return_(Exit::Trap);
        Ok(())
    }

    fn visit_xnor(
        &mut self,
        format: format::RRR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0 = self.rget(reg0);
        let src1 = self.rget(reg1);
        let src0_val = src0;
        let src1_val = src1;
        let xor_result = self.builder.ins().bxor(src0_val, src1_val);
        let result = self.builder.ins().bnot(xor_result);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_xor(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<(), Self::Error> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let src0_val = self.rget(reg0);
        let src1_val = self.rget(reg1);
        let result = self.builder.ins().bxor(src0_val, src1_val);
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_xor_imm(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let src_val = self.rget(reg1);
        let result = self.builder.ins().bxor_imm(src_val, imm0 as i64);
        self.rset(reg0, result);
        Ok(())
    }

    fn visit_zero_extend_16(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<(), Self::Error> {
        let format::RR { reg0, reg1 } = format;
        let src = self.rget(reg1);
        let result = self.builder.ins().band_imm(src, 0xFFFF);
        self.rset(reg0, result);
        Ok(())
    }
}
