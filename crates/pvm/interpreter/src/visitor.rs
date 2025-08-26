//! Instruction visitor for the pvm interpreter

use crate::{Interpreter, Result};
use core::ops::Range;
use parser::{
    format::{self, ISA},
    Visitor,
};
use pvm::Value;

impl Visitor for Interpreter {
    type Error = crate::Error;

    fn visit_add_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = (self.rget(reg0) as u32).wrapping_add(self.rget(reg1) as u32) as u64;

        self.rset(reg2, value.sign_ext32());
        Ok(())
    }

    fn visit_add_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0).wrapping_add(self.rget(reg1));
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_add_imm_32(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = (self.rget(reg1) as u32).wrapping_add(imm0 as u32) as u64;

        // sign extend the value if it is negative
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_add_imm_64(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.rget(reg1).wrapping_add(imm0);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_and(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0) & self.rget(reg1);
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_and_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.rget(reg1) & imm0;
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_and_inv(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0) & !self.rget(reg1);
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_branch_eq(&mut self, format: format::RRO, _range: &Range<usize>) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(off0, self.rget(reg0) == self.rget(reg1))
    }

    fn visit_branch_eq_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.rget(reg0) == imm0)
    }

    fn visit_branch_ge_s(&mut self, format: format::RRO, _range: &Range<usize>) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(off0, self.rget(reg0) as i64 >= self.rget(reg1) as i64)
    }

    fn visit_branch_ge_s_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.rget(reg0) as i64 >= imm0 as i64)
    }

    fn visit_branch_ge_u(&mut self, format: format::RRO, _range: &Range<usize>) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(off0, self.rget(reg0) >= self.rget(reg1))
    }

    fn visit_branch_ge_u_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.rget(reg0) >= imm0)
    }

    fn visit_branch_gt_s_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.rget(reg0) as i64 > imm0 as i64)
    }

    fn visit_branch_gt_u_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.rget(reg0) > imm0)
    }

    fn visit_branch_le_s_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.rget(reg0) as i64 <= imm0 as i64)
    }

    fn visit_branch_le_u_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.rget(reg0) <= imm0)
    }

    fn visit_branch_lt_s(&mut self, format: format::RRO, _range: &Range<usize>) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(off0, (self.rget(reg0) as i64) < (self.rget(reg1) as i64))
    }

    fn visit_branch_lt_u(&mut self, format: format::RRO, _range: &Range<usize>) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(off0, self.rget(reg0) < self.rget(reg1))
    }

    fn visit_branch_lt_s_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, (self.rget(reg0) as i64) < imm0 as i64)
    }

    fn visit_branch_lt_u_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.rget(reg0) < imm0)
    }

    fn visit_branch_ne(&mut self, format: format::RRO, _range: &Range<usize>) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(off0, self.rget(reg0) != self.rget(reg1))
    }

    fn visit_branch_ne_imm(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.rget(reg0) != imm0)
    }

    fn visit_cmov_iz(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if self.rget(reg1) == 0 {
            self.rset(reg2, self.rget(reg0));
        }
        Ok(())
    }

    fn visit_cmov_iz_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if self.rget(reg1) == 0 {
            self.rset(reg0, imm0);
        }
        Ok(())
    }

    fn visit_cmov_nz(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if self.rget(reg1) != 0 {
            self.rset(reg2, self.rget(reg0));
        }

        Ok(())
    }

    fn visit_cmov_nz_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if self.rget(reg1) != 0 {
            self.rset(reg0, imm0);
        }
        Ok(())
    }

    fn visit_count_set_bits_32(&mut self, format: format::RR, _range: &Range<usize>) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        let value = self.rget(reg1) as u32;
        self.rset(reg0, value.count_ones() as u64);
        Ok(())
    }

    fn visit_count_set_bits_64(&mut self, format: format::RR, _range: &Range<usize>) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        let value = self.rget(reg1);
        self.rset(reg0, value.count_ones() as u64);
        Ok(())
    }

    fn visit_div_u_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0) as u32;
        let divisor = self.rget(reg1) as u32;

        self.rset(
            reg2,
            if divisor == 0 {
                u64::MAX
            } else {
                (dividend.wrapping_div(divisor)) as u64
            }
            .sign_ext32(),
        );
        Ok(())
    }

    fn visit_div_u_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);

        self.rset(
            reg2,
            if divisor == 0 {
                u64::MAX
            } else {
                dividend.wrapping_div(divisor)
            },
        );
        Ok(())
    }

    fn visit_div_s_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0) as i32;
        let divisor = self.rget(reg1) as i32;

        self.rset(
            reg2,
            if divisor == 0 {
                u64::MAX
            } else if dividend == i32::MIN && divisor == -1 {
                i32::MIN as u64
            } else {
                ((dividend.wrapping_div(divisor)) as u32 as u64).sign_ext32()
            },
        );

        Ok(())
    }

    fn visit_div_s_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0) as i64;
        let divisor = self.rget(reg1) as i64;

        self.rset(
            reg2,
            if divisor == 0 {
                u64::MAX
            } else if dividend == i64::MIN && divisor == -1 {
                self.rget(reg0)
            } else {
                (dividend.wrapping_div(divisor)) as u64
            },
        );

        Ok(())
    }

    fn visit_ecalli(&mut self, format: format::I, _range: &Range<usize>) -> Result<()> {
        let format::I { imm0 } = format;
        Err(crate::Error::HostCall(imm0 as u32))
    }

    // Fallthrough instruction: no-op that allows execution to continue to the next instruction
    fn visit_fallthrough(&mut self, _range: &Range<usize>) -> Result<()> {
        Ok(())
    }

    fn visit_jump(&mut self, format: format::O, _range: &Range<usize>) -> Result<()> {
        let format::O { off0 } = format;
        self.branch(off0, true)
    }

    fn visit_jump_ind(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        self.djump(self.rget(reg0).wrapping_add(imm0) as u32)
    }

    fn visit_leading_zero_bits_32(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        let value = self.rget(reg1) as u32;
        self.rset(reg0, value.leading_zeros() as u64);
        Ok(())
    }

    fn visit_leading_zero_bits_64(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        let value = self.rget(reg1);
        self.rset(reg0, value.leading_zeros() as u64);
        Ok(())
    }

    fn visit_load_i8(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: i8 = self.read(imm0 as u32)?;
        self.rset(reg0, value.as_u64());
        Ok(())
    }

    fn visit_load_i16(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: i16 = self.read(imm0 as u32)?;
        self.rset(reg0, value.as_u64());
        Ok(())
    }

    fn visit_load_i32(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: i32 = self.read(imm0 as u32)?;
        self.rset(reg0, value.as_u64());
        Ok(())
    }

    fn visit_load_imm(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        self.rset(reg0, imm0);
        Ok(())
    }

    fn visit_load_imm_64(&mut self, format: format::REI, _range: &Range<usize>) -> Result<()> {
        let format::REI { reg0, eimm0 } = format;
        self.rset(reg0, eimm0);
        Ok(())
    }

    fn visit_load_imm_jump(&mut self, format: format::RIO, _range: &Range<usize>) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.rset(reg0, imm0);
        self.branch(off0, true)
    }

    fn visit_load_imm_jump_ind(
        &mut self,
        format: format::RRII,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RRII {
            reg0,
            reg1,
            imm0,
            imm1,
        } = format;

        let jump_address = self.rget(reg1).wrapping_add(imm1) as u32;
        self.rset(reg0, imm0);
        self.djump(jump_address)
    }

    fn visit_load_ind_i8(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value: i8 = self.read_offset(addr as u32, imm0 as u32)?;
        self.rset(reg0, value.as_u64());
        Ok(())
    }

    fn visit_load_ind_u8(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value: u8 = self.read_offset(addr as u32, imm0 as u32)?;
        self.rset(reg0, value as u64);
        Ok(())
    }

    fn visit_load_ind_u16(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value: u16 = self.read_offset(addr as u32, imm0 as u32)?;
        self.rset(reg0, value as u64);
        Ok(())
    }

    fn visit_load_ind_i16(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value: i16 = self.read_offset(addr as u32, imm0 as u32)?;
        self.rset(reg0, value as u64);
        Ok(())
    }

    fn visit_load_ind_u32(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value: u32 = self.read_offset(addr as u32, imm0 as u32)?;
        self.rset(reg0, value as u64);
        Ok(())
    }

    fn visit_load_ind_i32(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value: i32 = self.read_offset(addr as u32, imm0 as u32)?;
        self.rset(reg0, value.as_u64());
        Ok(())
    }

    fn visit_load_ind_u64(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.rget(reg1);
        let value: u64 = self.read_offset(addr as u32, imm0 as u32)?;
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_load_u8(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: u8 = self.read(imm0 as u32)?;
        self.rset(reg0, value.as_u64());
        Ok(())
    }

    fn visit_load_u16(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: u16 = self.read(imm0 as u32)?;
        self.rset(reg0, value.as_u64());
        Ok(())
    }

    fn visit_load_u32(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: u32 = self.read(imm0 as u32)?;
        self.rset(reg0, value.as_u64());
        Ok(())
    }

    fn visit_load_u64(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: u64 = self.read(imm0 as u32)?;
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_max(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = (self.rget(reg0) as i64).max(self.rget(reg1) as i64);
        self.rset(reg2, value as u64);
        Ok(())
    }

    fn visit_max_u(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0).max(self.rget(reg1));
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_min(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = (self.rget(reg0) as i64).min(self.rget(reg1) as i64);
        self.rset(reg2, value as u64);
        Ok(())
    }

    fn visit_min_u(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0).min(self.rget(reg1));
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_move_reg(&mut self, format: format::RR, _range: &Range<usize>) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        self.rset(reg0, self.rget(reg1));
        Ok(())
    }

    fn visit_mul_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = (self.rget(reg0) as u32).wrapping_mul(self.rget(reg1) as u32) as u64;
        self.rset(reg2, value.sign_ext32());
        Ok(())
    }

    fn visit_mul_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0).wrapping_mul(self.rget(reg1));
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_mul_imm_32(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = (self.rget(reg1) as u32).wrapping_mul(imm0 as u32) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_mul_imm_64(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.rget(reg1).wrapping_mul(imm0);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_mul_upper_s_s(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let a = self.rget(reg0) as i64;
        let b = self.rget(reg1) as i64;
        let result = ((a as i128).wrapping_mul(b as i128) >> 64) as u64;
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_mul_upper_u_u(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let a = self.rget(reg0);
        let b = self.rget(reg1);
        let result = ((a as u128 * b as u128) >> 64) as u64;
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_mul_upper_s_u(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let a = self.rget(reg0) as i64;
        let b = self.rget(reg1);
        let result = ((a as i128).wrapping_mul(b as i128) >> 64) as u64;
        self.rset(reg2, result);
        Ok(())
    }

    fn visit_neg_add_imm_32(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = (imm0 as u32).wrapping_sub(self.rget(reg1) as u32) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_neg_add_imm_64(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = imm0.wrapping_sub(self.rget(reg1));
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_or(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0) | self.rget(reg1);
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_or_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.rget(reg1) | imm0;
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_or_inv(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0) | !self.rget(reg1);
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_rem_u_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0) as u32;
        let divisor = self.rget(reg1) as u32;

        self.rset(
            reg2,
            if divisor == 0 {
                (dividend as u64).sign_ext32()
            } else {
                (dividend.wrapping_rem(divisor)) as u64
            }
            .sign_ext32(),
        );
        Ok(())
    }

    fn visit_rem_u_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0);
        let divisor = self.rget(reg1);

        self.rset(
            reg2,
            if divisor == 0 {
                dividend
            } else {
                dividend.wrapping_rem(divisor)
            },
        );
        Ok(())
    }

    fn visit_rem_s_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0) as i32;
        let divisor = self.rget(reg1) as i32;

        self.rset(
            reg2,
            if divisor == 0 {
                self.rget(reg0).sign_ext32()
            } else if dividend == i32::MIN && divisor == -1 {
                0
            } else {
                ((dividend.wrapping_rem(divisor) as u32) as u64).sign_ext32()
            },
        );
        Ok(())
    }

    fn visit_rem_s_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.rget(reg0) as i64;
        let divisor = self.rget(reg1) as i64;

        self.rset(
            reg2,
            if divisor == 0 {
                self.rget(reg0)
            } else if dividend == i64::MIN && divisor == -1 {
                0
            } else {
                (dividend.wrapping_rem(divisor)) as u64
            },
        );
        Ok(())
    }

    fn visit_reverse_bytes(&mut self, format: format::RR, _range: &Range<usize>) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        let mut value = self.rget(reg1).to_le_bytes();
        value.reverse();

        self.rset(reg0, u64::from_le_bytes(value));
        Ok(())
    }

    fn visit_rot_l_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let rotation = self.rget(reg1) % 32;
        let value = ((self.rget(reg0) as u32).rotate_left(rotation as u32)) as u64;
        self.rset(reg2, value.sign_ext32());
        Ok(())
    }

    fn visit_rot_l_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let rotation = self.rget(reg1) % 64;
        let value = self.rget(reg0).rotate_left(rotation as u32);
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_rot_r_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let rotation = self.rget(reg1) % 32;
        let value = ((self.rget(reg0) as u32).rotate_right(rotation as u32)) as u64;
        self.rset(reg2, value.sign_ext32());
        Ok(())
    }

    fn visit_rot_r_32_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let rotation = imm0 % 32;
        let value = ((self.rget(reg1) as u32).rotate_right(rotation as u32)) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_rot_r_32_imm_alt(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let rotation = self.rget(reg1) % 32;
        let value = ((imm0 as u32).rotate_right(rotation as u32)) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_rot_r_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let rotation = self.rget(reg1) % 64;
        let value = self.rget(reg0).rotate_right(rotation as u32);
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_rot_r_64_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let rotation = imm0 % 64;
        let value = self.rget(reg1).rotate_right(rotation as u32);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_rot_r_64_imm_alt(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let rotation = self.rget(reg1) % 64;
        let value = imm0.rotate_right(rotation as u32);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_sbrk(&mut self, format: format::RR, _range: &Range<usize>) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        let increment = self.rget(reg1);
        self.rset(reg0, self.memory.heap_ptr as u64);
        if increment == 0 {
            return Ok(());
        }

        let funp = |x: u64| x.div_ceil(parser::PAGE_SIZE) * parser::PAGE_SIZE;
        let boundary = funp(self.memory.heap_ptr as u64);
        let nptr = self.memory.heap_ptr as u64 + increment;
        if nptr > boundary {
            let start = boundary / parser::PAGE_SIZE;
            let count = funp(nptr) / parser::PAGE_SIZE - start;
            self.allocate(start as u32, count as u32)?;
        }

        self.memory.heap_ptr += increment as u32;
        Ok(())
    }

    fn visit_set_gt_s_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if (self.rget(reg1) as i64) > (imm0 as i64) {
            self.rset(reg0, 1);
        } else {
            self.rset(reg0, 0);
        }

        Ok(())
    }

    fn visit_set_gt_u_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if self.rget(reg1) > imm0 {
            self.rset(reg0, 1);
        } else {
            self.rset(reg0, 0);
        }
        Ok(())
    }

    fn visit_set_lt_s_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if (self.rget(reg1) as i64) < (imm0 as i64) {
            self.rset(reg0, 1);
        } else {
            self.rset(reg0, 0);
        }

        Ok(())
    }

    fn visit_set_lt_u_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if self.rget(reg1) < imm0 {
            self.rset(reg0, 1);
        } else {
            self.rset(reg0, 0);
        }

        Ok(())
    }

    fn visit_set_lt_u(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if self.rget(reg0) < self.rget(reg1) {
            self.rset(reg2, 1);
        } else {
            self.rset(reg2, 0);
        }

        Ok(())
    }

    fn visit_set_lt_s(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if (self.rget(reg0) as i64) < (self.rget(reg1) as i64) {
            self.rset(reg2, 1);
        } else {
            self.rset(reg2, 0);
        }

        Ok(())
    }

    fn visit_shar_r_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.rget(reg1) % 32;
        let value = ((self.rget(reg0) as i32).wrapping_shr(shift as u32)) as u64;
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_shar_r_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.rget(reg1) % 64;
        let value = ((self.rget(reg0) as i64) >> shift) as u64;
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_shar_r_imm_32(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 32;
        let value = ((self.rget(reg1) as i32).wrapping_shr(shift as u32)) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_shar_r_imm_64(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 64;
        let value = ((self.rget(reg1) as i64) >> shift) as u64;
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_shar_r_imm_alt_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1) % 32;
        let value = ((imm0 as i32).wrapping_shr(shift as u32)) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_shar_r_imm_alt_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1) % 64;
        let value = ((imm0 as i64).wrapping_shr(shift as u32)) as u64;
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_shlo_l_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.rget(reg1) % 32;
        let value = (self.rget(reg0) as u32).wrapping_shl(shift as u32) as u64;
        self.rset(reg2, value.sign_ext32());
        Ok(())
    }

    fn visit_shlo_l_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.rget(reg1) % 64;
        let value = self.rget(reg0).wrapping_shl(shift as u32);
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_shlo_l_imm_32(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 32;
        let value = (self.rget(reg1) as u32).wrapping_shl(shift as u32) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_shlo_l_imm_64(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 64;
        let value = self.rget(reg1) << shift;
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_shlo_l_imm_alt_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1) % 32;
        let value = ((imm0 as u32).wrapping_shl(shift as u32)) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_shlo_l_imm_alt_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1) % 64;
        let value = imm0.wrapping_shl(shift as u32);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_shlo_r_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.rget(reg1) % 32;
        let value = (self.rget(reg0) as u32).wrapping_shr(shift as u32) as u64;
        self.rset(reg2, value.sign_ext32());
        Ok(())
    }

    fn visit_shlo_r_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.rget(reg1) % 64;
        let value = self.rget(reg0).wrapping_shr(shift as u32);
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_shlo_r_imm_32(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 32;
        let value = (self.rget(reg1) as u32).wrapping_shr(shift as u32) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_shlo_r_imm_64(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 64;
        let value = self.rget(reg1).wrapping_shr(shift as u32);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_shlo_r_imm_alt_32(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = (self.rget(reg1) as u32) % 32;
        let value = ((imm0 as u32).wrapping_shr(shift)) as u64;
        self.rset(reg0, value.sign_ext32());
        Ok(())
    }

    fn visit_shlo_r_imm_alt_64(
        &mut self,
        format: format::RRI,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.rget(reg1) % 64;
        let value = imm0.wrapping_shr(shift as u32);
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_sign_extend_8(&mut self, format: format::RR, _range: &Range<usize>) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        self.rset(reg0, self.rget(reg1) as i8 as u64);
        Ok(())
    }

    fn visit_sign_extend_16(&mut self, format: format::RR, _range: &Range<usize>) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        self.rset(reg0, self.rget(reg1) as i16 as u64);
        Ok(())
    }

    fn visit_store_u8(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value = self.rget(reg0) as u8;
        self.write(imm0 as u32, value)
    }

    fn visit_store_u16(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value = self.rget(reg0) as u16;
        self.write(imm0 as u32, value)
    }

    fn visit_store_u32(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value = self.rget(reg0) as u32;
        self.write(imm0 as u32, value)
    }

    fn visit_store_u64(&mut self, format: format::RI, _range: &Range<usize>) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value = self.rget(reg0);
        self.write(imm0 as u32, value)
    }

    fn visit_store_imm_u8(&mut self, format: format::II, _range: &Range<usize>) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        let value = imm1 as u8;
        self.write(imm0 as u32, value)
    }

    fn visit_store_imm_u16(&mut self, format: format::II, _range: &Range<usize>) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        let value = imm1 as u16;
        self.write(imm0 as u32, value)
    }

    fn visit_store_imm_u32(&mut self, format: format::II, _range: &Range<usize>) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        let value = imm1 as u32;
        self.write(imm0 as u32, value)
    }

    fn visit_store_imm_u64(&mut self, format: format::II, _range: &Range<usize>) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        let value = imm1;
        self.write(imm0 as u32, value)
    }

    fn visit_store_imm_ind_u8(&mut self, format: format::RII, _range: &Range<usize>) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        let address = self.rget(reg0);
        self.write_offset(address as u32, imm0 as u32, imm1 as u8)
    }

    fn visit_store_imm_ind_u16(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        let address = self.rget(reg0);
        self.write_offset(address as u32, imm0 as u32, imm1 as u16)
    }

    fn visit_store_imm_ind_u32(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        let address = self.rget(reg0);
        self.write_offset(address as u32, imm0 as u32, imm1 as u32)
    }

    fn visit_store_imm_ind_u64(
        &mut self,
        format: format::RII,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        let address = self.rget(reg0);
        self.write_offset(address as u32, imm0 as u32, imm1)
    }

    fn visit_store_ind_u8(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let address = self.rget(reg1);
        self.write_offset(address as u32, imm0 as u32, self.rget(reg0) as u8)
    }

    fn visit_store_ind_u16(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let address = self.rget(reg1);
        self.write_offset(address as u32, imm0 as u32, self.rget(reg0) as u16)
    }

    fn visit_store_ind_u32(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let address = self.rget(reg1);
        self.write_offset(address as u32, imm0 as u32, self.rget(reg0) as u32)
    }

    fn visit_store_ind_u64(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let address = self.rget(reg1);
        self.write_offset(address as u32, imm0 as u32, self.rget(reg0))
    }

    fn visit_sub_32(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = (self.rget(reg0) as u32).wrapping_sub(self.rget(reg1) as u32) as u64;

        self.rset(reg2, value.sign_ext32());
        Ok(())
    }

    fn visit_sub_64(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0).wrapping_sub(self.rget(reg1));
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_trap(&mut self, _range: &Range<usize>) -> Result<()> {
        Err(crate::Error::Trap(false))
    }

    fn visit_trailing_zero_bits_32(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        let value = self.rget(reg1) as u32;
        self.rset(reg0, value.trailing_zeros() as u64);
        Ok(())
    }

    fn visit_trailing_zero_bits_64(
        &mut self,
        format: format::RR,
        _range: &Range<usize>,
    ) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        let value = self.rget(reg1);
        self.rset(reg0, value.trailing_zeros() as u64);
        Ok(())
    }

    fn visit_xnor(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = !(self.rget(reg0) ^ self.rget(reg1));
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_xor(&mut self, format: format::RRR, _range: &Range<usize>) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.rget(reg0) ^ self.rget(reg1);
        self.rset(reg2, value);
        Ok(())
    }

    fn visit_xor_imm(&mut self, format: format::RRI, _range: &Range<usize>) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.rget(reg1) ^ imm0;
        self.rset(reg0, value);
        Ok(())
    }

    fn visit_zero_extend_16(&mut self, format: format::RR, _range: &Range<usize>) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        self.rset(reg0, self.rget(reg1) as u16 as u64);
        Ok(())
    }
}
