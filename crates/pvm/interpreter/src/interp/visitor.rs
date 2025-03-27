//! Instruction visitor for the pvm interpreter

use crate::{interp::Interpreter, Result, Value};
use pvm_parser::{
    format::{self, ISA},
    Visitor,
};

impl Visitor for Interpreter {
    type Error = crate::Error;

    fn visit_trap(&mut self) -> Result<()> {
        Err(crate::Error::Trap(false))
    }

    fn visit_add_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = (self.registers[reg0 as usize] as u32)
            .wrapping_add(self.registers[reg1 as usize] as u32) as u64;

        self.registers[reg2 as usize] = value.sign_ext32();
        Ok(())
    }

    fn visit_add_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].wrapping_add(self.registers[reg1 as usize]);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_add_imm_32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = (self.registers[reg1 as usize] as u32).wrapping_add(imm0 as u32) as u64;

        // sign extend the value if it is negative
        self.registers[reg0 as usize] = value.sign_ext32();
        Ok(())
    }

    fn visit_add_imm_64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.registers[reg1 as usize].wrapping_add(imm0);
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_and(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize] & self.registers[reg1 as usize];
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_and_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.registers[reg1 as usize] & imm0;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_and_inv(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize] & !self.registers[reg1 as usize];
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_branch_eq(&mut self, format: format::RRO) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(
            off0,
            self.registers[reg0 as usize] == self.registers[reg1 as usize],
        )
    }

    fn visit_branch_eq_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.registers[reg0 as usize] == imm0)
    }

    fn visit_branch_ge_s(&mut self, format: format::RRO) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(
            off0,
            self.registers[reg0 as usize] as i64 >= self.registers[reg1 as usize] as i64,
        )
    }

    fn visit_branch_ge_s_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.registers[reg0 as usize] as i64 >= imm0 as i64)
    }

    fn visit_branch_ge_u(&mut self, format: format::RRO) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(
            off0,
            self.registers[reg0 as usize] >= self.registers[reg1 as usize],
        )
    }

    fn visit_branch_ge_u_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.registers[reg0 as usize] >= imm0)
    }

    fn visit_branch_gt_s_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.registers[reg0 as usize] as i32 > imm0 as i32)
    }

    fn visit_branch_gt_u_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.registers[reg0 as usize] > imm0)
    }

    fn visit_branch_le_s_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.registers[reg0 as usize] as i32 <= imm0 as i32)
    }

    fn visit_branch_le_u_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.registers[reg0 as usize] <= imm0)
    }

    fn visit_branch_lt_s(&mut self, format: format::RRO) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(
            off0,
            (self.registers[reg0 as usize] as i64) < (self.registers[reg1 as usize] as i64),
        )
    }

    fn visit_branch_lt_u(&mut self, format: format::RRO) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(
            off0,
            self.registers[reg0 as usize] < self.registers[reg1 as usize],
        )
    }

    fn visit_branch_lt_s_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, (self.registers[reg0 as usize] as i64) < imm0 as i64)
    }

    fn visit_branch_lt_u_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.registers[reg0 as usize] < imm0)
    }

    fn visit_branch_ne(&mut self, format: format::RRO) -> Result<()> {
        let format::RRO { reg0, reg1, off0 } = format;
        self.branch(
            off0,
            self.registers[reg0 as usize] != self.registers[reg1 as usize],
        )
    }

    fn visit_branch_ne_imm(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.branch(off0, self.registers[reg0 as usize] != imm0)
    }

    fn visit_cmov_iz(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if self.registers[reg1 as usize] == 0 {
            self.registers[reg2 as usize] = self.registers[reg0 as usize];
        }
        Ok(())
    }

    fn visit_cmov_iz_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if self.registers[reg1 as usize] == 0 {
            self.registers[reg0 as usize] = imm0;
        }
        Ok(())
    }

    fn visit_cmov_nz(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if self.registers[reg1 as usize] != 0 {
            self.registers[reg2 as usize] = self.registers[reg0 as usize];
        }
        Ok(())
    }

    fn visit_cmov_nz_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if self.registers[reg1 as usize] != 0 {
            self.registers[reg0 as usize] = imm0;
        }
        Ok(())
    }

    fn visit_div_u_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize] as u32;
        let divisor = self.registers[reg1 as usize] as u32;

        self.registers[reg2 as usize] = if divisor == 0 {
            u64::MAX
        } else {
            (dividend.wrapping_div(divisor)) as u64
        };
        Ok(())
    }

    fn visit_div_u_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize];
        let divisor = self.registers[reg1 as usize];

        self.registers[reg2 as usize] = if divisor == 0 {
            u64::MAX
        } else {
            dividend.wrapping_div(divisor)
        };
        Ok(())
    }

    fn visit_div_s_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize] as i32;
        let divisor = self.registers[reg1 as usize] as i32;

        self.registers[reg2 as usize] = if divisor == 0 {
            u64::MAX
        } else if dividend == i32::MIN && divisor == -1 {
            i32::MIN as u64
        } else {
            ((dividend.wrapping_div(divisor)) as u32 as u64).sign_ext32()
        };

        Ok(())
    }

    fn visit_div_s_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize] as i64;
        let divisor = self.registers[reg1 as usize] as i64;

        self.registers[reg2 as usize] = if divisor == 0 {
            u64::MAX
        } else if dividend == i64::MIN && divisor == -1 {
            self.registers[reg0 as usize]
        } else {
            (dividend.wrapping_div(divisor)) as u64
        };

        Ok(())
    }

    // TODO: re-check the fallthrough logic
    fn visit_fallthrough(&mut self) -> Result<()> {
        Ok(())
    }

    fn visit_jump(&mut self, format: format::O) -> Result<()> {
        let format::O { off0 } = format;
        self.branch(off0, true)
    }

    fn visit_jump_ind(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        self.djump(self.registers[reg0 as usize].wrapping_add(imm0) as u32)
    }

    fn visit_load_i8(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: i8 = self.memory.read(imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_i16(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: i16 = self.memory.read(imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_i32(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: i32 = self.memory.read(imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_imm(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        self.registers[reg0 as usize] = imm0;
        Ok(())
    }

    fn visit_load_imm_64(&mut self, format: format::REI) -> Result<()> {
        let format::REI { reg0, eimm0 } = format;
        self.registers[reg0 as usize] = eimm0;
        Ok(())
    }

    fn visit_load_imm_jump(&mut self, format: format::RIO) -> Result<()> {
        let format::RIO { reg0, off0, imm0 } = format;
        self.registers[reg0 as usize] = imm0;
        self.jump = Some(self.pc.wrapping_add(off0 as usize));
        Ok(())
    }

    fn visit_load_imm_jump_ind(&mut self, format: format::RRII) -> Result<()> {
        let format::RRII {
            reg0,
            reg1,
            imm0,
            imm1,
        } = format;

        let result = self.djump(self.registers[reg1 as usize].wrapping_add(imm1) as u32);
        self.registers[reg0 as usize] = imm0;
        result
    }

    fn visit_load_ind_i8(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.registers[reg1 as usize];
        let value: i8 = self.memory.read_offset(addr, imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_ind_u8(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.registers[reg1 as usize];
        let value: u8 = self.memory.read_offset(addr, imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_ind_u16(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.registers[reg1 as usize];
        let value: u16 = self.memory.read_offset(addr, imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_ind_i16(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.registers[reg1 as usize];
        let value: i16 = self.memory.read_offset(addr, imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_ind_u32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.registers[reg1 as usize];
        let value: u32 = self.memory.read_offset(addr, imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_ind_i32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.registers[reg1 as usize];
        let value: i32 = self.memory.read_offset(addr, imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_ind_u64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let addr = self.registers[reg1 as usize];
        let value: u64 = self.memory.read_offset(addr, imm0)?;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_load_u8(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: u8 = self.memory.read(imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_u16(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: u16 = self.memory.read(imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_u32(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: u32 = self.memory.read(imm0)?;
        self.registers[reg0 as usize] = value.as_u64();
        Ok(())
    }

    fn visit_load_u64(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value: u64 = self.memory.read(imm0)?;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_max(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].max(self.registers[reg1 as usize]);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_max_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].max(self.registers[reg1 as usize]);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_min(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].min(self.registers[reg1 as usize]);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_min_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].min(self.registers[reg1 as usize]);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_move_reg(&mut self, format: format::RR) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        self.registers[reg0 as usize] = self.registers[reg1 as usize];
        Ok(())
    }

    fn visit_mul_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = (self.registers[reg0 as usize] as u32)
            .wrapping_mul(self.registers[reg1 as usize] as u32) as u64;

        // sign extend the value if it is negative
        self.registers[reg2 as usize] = value.sign_ext32();
        Ok(())
    }

    fn visit_mul_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].wrapping_mul(self.registers[reg1 as usize]);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_mul_imm_32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = (self.registers[reg1 as usize] as u32).wrapping_mul(imm0 as u32) as u64;

        // sign extend the value if it is negative
        self.registers[reg0 as usize] = value.sign_ext32();
        Ok(())
    }

    fn visit_mul_imm_64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.registers[reg1 as usize].wrapping_mul(imm0);
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_mul_upper_s_s(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let a = self.registers[reg0 as usize] as i32 as i64;
        let b = self.registers[reg1 as usize] as i32 as i64;
        let result = ((a * b) >> 32) as u64;
        self.registers[reg2 as usize] = result;
        Ok(())
    }

    fn visit_mul_upper_u_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let a = self.registers[reg0 as usize];
        let b = self.registers[reg1 as usize];
        let result = (a * b) >> 32;
        self.registers[reg2 as usize] = result;
        Ok(())
    }

    fn visit_mul_upper_s_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let a = self.registers[reg0 as usize] as i32 as i64;
        let b = self.registers[reg1 as usize];
        let result = ((a * b as i64) >> 32) as u64;
        self.registers[reg2 as usize] = result;
        Ok(())
    }

    fn visit_neg_add_imm_32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = (imm0 as u32).wrapping_sub(self.registers[reg1 as usize] as u32) as u64;
        self.registers[reg0 as usize] = value.sign_ext32();
        Ok(())
    }

    fn visit_neg_add_imm_64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = imm0.wrapping_sub(self.registers[reg1 as usize]);
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_or(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize] | self.registers[reg1 as usize];
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_or_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.registers[reg1 as usize] | imm0;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_or_inv(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize] | !self.registers[reg1 as usize];
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_rem_u_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize] as u32;
        let divisor = self.registers[reg1 as usize] as u32;

        self.registers[reg2 as usize] = if divisor == 0 {
            (dividend as u64).sign_ext32()
        } else {
            (dividend.wrapping_rem(divisor)) as u64
        };
        Ok(())
    }

    fn visit_rem_u_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize];
        let divisor = self.registers[reg1 as usize];

        self.registers[reg2 as usize] = if divisor == 0 {
            dividend
        } else {
            dividend.wrapping_rem(divisor)
        };
        Ok(())
    }

    fn visit_rem_s_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize] as i32;
        let divisor = self.registers[reg1 as usize] as i32;

        self.registers[reg2 as usize] = if divisor == 0 {
            self.registers[reg0 as usize].sign_ext32()
        } else if dividend == i32::MIN && divisor == -1 {
            0
        } else {
            ((dividend.wrapping_rem(divisor) as u32) as u64).sign_ext32()
        };
        Ok(())
    }

    fn visit_rem_s_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize] as i64;
        let divisor = self.registers[reg1 as usize] as i64;

        self.registers[reg2 as usize] = if divisor == 0 {
            self.registers[reg0 as usize]
        } else if dividend == i64::MIN && divisor == -1 {
            0
        } else {
            (dividend.wrapping_rem(divisor)) as u64
        };
        Ok(())
    }

    fn visit_reverse_bytes(&mut self, format: format::RR) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        let mut value = self.registers[reg0 as usize].to_le_bytes();
        value.reverse();

        self.registers[reg1 as usize] = u64::from_le_bytes(value);
        Ok(())
    }

    fn visit_rot_l_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].rotate_left(self.registers[reg1 as usize] as u32);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_rot_l_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].rotate_left(self.registers[reg1 as usize] as u32);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_rot_r_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value =
            self.registers[reg0 as usize].rotate_right(self.registers[reg1 as usize] as u32);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_rot_r_32_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.registers[reg0 as usize].rotate_right(imm0 as u32);
        self.registers[reg1 as usize] = value;
        Ok(())
    }

    // TODO: fix this with u64
    fn visit_rot_r_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value =
            self.registers[reg0 as usize].rotate_right(self.registers[reg1 as usize] as u32);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_rot_r_64_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.registers[reg0 as usize].rotate_right(imm0 as u32);
        self.registers[reg1 as usize] = value;
        Ok(())
    }

    fn visit_set_gt_s_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if (self.registers[reg1 as usize] as i32) > (imm0 as i32) {
            self.registers[reg0 as usize] = 1;
        } else {
            self.registers[reg0 as usize] = 0;
        }

        Ok(())
    }

    fn visit_set_gt_u_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if self.registers[reg1 as usize] > imm0 {
            self.registers[reg0 as usize] = 1;
        } else {
            self.registers[reg0 as usize] = 0;
        }
        Ok(())
    }

    fn visit_set_lt_s_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if (self.registers[reg1 as usize] as i32) < (imm0 as i32) {
            self.registers[reg0 as usize] = 1;
        } else {
            self.registers[reg0 as usize] = 0;
        }

        Ok(())
    }

    fn visit_set_lt_u_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        if self.registers[reg1 as usize] < imm0 {
            self.registers[reg0 as usize] = 1;
        } else {
            self.registers[reg0 as usize] = 0;
        }

        Ok(())
    }

    fn visit_set_lt_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if self.registers[reg0 as usize] < self.registers[reg1 as usize] {
            self.registers[reg2 as usize] = 1;
        } else {
            self.registers[reg2 as usize] = 0;
        }

        Ok(())
    }

    fn visit_set_lt_s(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        if (self.registers[reg0 as usize] as i32) < (self.registers[reg1 as usize] as i32) {
            self.registers[reg2 as usize] = 1;
        } else {
            self.registers[reg2 as usize] = 0;
        }

        Ok(())
    }

    fn visit_shar_r_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.registers[reg1 as usize] % 32;
        let value = ((self.registers[reg0 as usize] as i32).wrapping_shr(shift as u32)) as u64;
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_shar_r_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.registers[reg1 as usize] % 64;
        let value = ((self.registers[reg0 as usize] as i64) >> shift) as u64;
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_shar_r_imm_32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 32;
        let value = ((self.registers[reg1 as usize] as i32).wrapping_shr(shift as u32)) as u64;
        self.registers[reg0 as usize] = value.sign_ext32();
        Ok(())
    }

    fn visit_shar_r_imm_64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 64;
        let value = ((self.registers[reg1 as usize] as i64) >> shift) as u64;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shar_r_imm_alt_32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.registers[reg1 as usize] % 32;
        let value = ((imm0 as i32).wrapping_shr(shift as u32)) as u64;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shar_r_imm_alt_64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.registers[reg1 as usize] % 64;
        let value = ((imm0 as i64).wrapping_shr(shift as u32)) as u64;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shlo_l_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.registers[reg1 as usize] % 32;
        let value = (self.registers[reg0 as usize] as u32).wrapping_shl(shift as u32) as u64;
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_shlo_l_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.registers[reg1 as usize] % 64;
        let value = self.registers[reg0 as usize].wrapping_shl(shift as u32);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_shlo_l_imm_32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 32;
        let value = (self.registers[reg1 as usize] as u32).wrapping_shl(shift as u32) as u64;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shlo_l_imm_64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 64;
        let value = self.registers[reg1 as usize] << shift;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shlo_l_imm_alt_32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.registers[reg1 as usize] % 32;
        let value = imm0.wrapping_shl(shift as u32);
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shlo_l_imm_alt_64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.registers[reg1 as usize] % 64;
        let value = imm0.wrapping_shl(shift as u32);
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shlo_r_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.registers[reg1 as usize] % 32;
        let value = (self.registers[reg0 as usize] as u32).wrapping_shr(shift as u32) as u64;
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_shlo_r_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.registers[reg1 as usize] % 32;
        let value = self.registers[reg0 as usize].wrapping_shr(shift as u32);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_shlo_r_imm_32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 32;
        let value = (self.registers[reg1 as usize] as u32).wrapping_shr(shift as u32) as u64;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shlo_r_imm_64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = imm0 % 64;
        let value = self.registers[reg1 as usize].wrapping_shr(shift as u32);
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shlo_r_imm_alt_32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = (self.registers[reg1 as usize] as u32) % 32;
        let value = ((imm0 as u32).wrapping_shr(shift)) as u64;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_shlo_r_imm_alt_64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let shift = self.registers[reg1 as usize] % 64;
        let value = imm0.wrapping_shr(shift as u32);
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_sign_extend_8(&mut self, format: format::RR) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        self.registers[reg0 as usize] = self.registers[reg1 as usize] as i8 as u64;
        Ok(())
    }

    fn visit_sign_extend_16(&mut self, format: format::RR) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        self.registers[reg0 as usize] = self.registers[reg1 as usize] as i16 as u64;
        Ok(())
    }

    fn visit_store_u8(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value = self.registers[reg0 as usize] as u8;
        self.memory.write(imm0, value)
    }

    fn visit_store_u16(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value = self.registers[reg0 as usize] as u16;
        self.memory.write(imm0, value)
    }

    fn visit_store_u32(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value = self.registers[reg0 as usize] as u32;
        self.memory.write(imm0, value)
    }

    fn visit_store_u64(&mut self, format: format::RI) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        let value = self.registers[reg0 as usize];
        self.memory.write(imm0, value)
    }

    fn visit_store_imm_u8(&mut self, format: format::II) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        let value = imm1 as u8;
        self.memory.write(imm0, value)
    }

    fn visit_store_imm_u16(&mut self, format: format::II) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        let value = imm1 as u16;
        self.memory.write(imm0, value)
    }

    fn visit_store_imm_u32(&mut self, format: format::II) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        let value = imm1 as u32;
        self.memory.write(imm0, value)
    }

    fn visit_store_imm_u64(&mut self, format: format::II) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        let value = imm1;
        self.memory.write(imm0, value)
    }

    fn visit_store_imm_ind_u8(&mut self, format: format::RII) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        let address = self.registers[reg0 as usize];
        self.memory.write_offset(address, imm0, imm1 as u8)
    }

    fn visit_store_imm_ind_u16(&mut self, format: format::RII) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        let address = self.registers[reg0 as usize];
        self.memory.write_offset(address, imm0, imm1 as u16)
    }

    fn visit_store_imm_ind_u32(&mut self, format: format::RII) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        let address = self.registers[reg0 as usize];
        self.memory.write_offset(address, imm0, imm1 as u32)
    }

    fn visit_store_imm_ind_u64(&mut self, format: format::RII) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        let address = self.registers[reg0 as usize];
        self.memory.write_offset(address, imm0, imm1)
    }

    fn visit_store_ind_u8(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let address = self.registers[reg1 as usize];
        self.memory
            .write_offset(address, imm0, self.registers[reg0 as usize] as u8)
    }

    fn visit_store_ind_u16(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let address = self.registers[reg1 as usize];
        self.memory
            .write_offset(address, imm0, self.registers[reg0 as usize] as u16)
    }

    fn visit_store_ind_u32(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let address = self.registers[reg1 as usize];
        self.memory
            .write_offset(address, imm0, self.registers[reg0 as usize] as u32)
    }

    fn visit_store_ind_u64(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let address = self.registers[reg1 as usize];
        self.memory
            .write_offset(address, imm0, self.registers[reg0 as usize])
    }

    fn visit_sub_32(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = (self.registers[reg0 as usize] as u32)
            .wrapping_sub(self.registers[reg1 as usize] as u32) as u64;

        self.registers[reg2 as usize] = value.sign_ext32();
        Ok(())
    }

    fn visit_sub_64(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].wrapping_sub(self.registers[reg1 as usize]);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_xnor(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize] ^ self.registers[reg1 as usize];
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_xor(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize] ^ self.registers[reg1 as usize];
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_xor_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.registers[reg1 as usize] ^ imm0;
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_zero_extend_16(&mut self, format: format::RR) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        self.registers[reg0 as usize] = self.registers[reg1 as usize] as u16 as u64;
        Ok(())
    }
}
