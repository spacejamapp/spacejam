//! Instruction visitor for the pvm interpreter

use crate::{interp::Interpreter, status::Status};
use anyhow::Result;
use parser::{format, Visitor};

impl Visitor for Interpreter {
    fn visit_trap(&mut self) -> Result<()> {
        self.status = Status::Trap;
        Ok(())
    }

    fn visit_add(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].wrapping_add(self.registers[reg1 as usize]);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_add_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.registers[reg1 as usize].wrapping_add(imm0);
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_div_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize];
        let divisor = self.registers[reg1 as usize];

        self.registers[reg2 as usize] = if divisor == 0 {
            u32::MAX // 2^32 - 1
        } else {
            dividend.wrapping_div(divisor)
        };
        Ok(())
    }

    fn visit_div_s(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize] as i32;
        let divisor = self.registers[reg1 as usize] as i32;

        self.registers[reg2 as usize] = if divisor == 0 {
            u32::MAX // 2^32 - 1
        } else if dividend == i32::MIN && divisor == -1 {
            self.registers[reg0 as usize] // Return dividend unchanged
        } else {
            (dividend.wrapping_div(divisor)) as u32
        };
        Ok(())
    }

    fn visit_mul(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].wrapping_mul(self.registers[reg1 as usize]);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_mul_imm(&mut self, format: format::RRI) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        let value = self.registers[reg1 as usize].wrapping_mul(imm0);
        self.registers[reg0 as usize] = value;
        Ok(())
    }

    fn visit_neg_add_imm(&mut self, format: format::RRI) -> Result<()> {
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

    fn visit_rem_s(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize] as i32;
        let divisor = self.registers[reg1 as usize] as i32;
        self.registers[reg2 as usize] = if divisor == 0 {
            self.registers[reg0 as usize] // Return dividend when divisor is 0
        } else if dividend == i32::MIN && divisor == -1 {
            0 // Special case as per spec
        } else {
            (dividend.wrapping_rem(divisor)) as u32
        };
        Ok(())
    }

    fn visit_rem_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let dividend = self.registers[reg0 as usize];
        let divisor = self.registers[reg1 as usize];
        self.registers[reg2 as usize] = if divisor == 0 {
            dividend // Return dividend when divisor is 0
        } else {
            dividend.wrapping_rem(divisor)
        };
        Ok(())
    }

    fn visit_sub(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = self.registers[reg0 as usize].wrapping_sub(self.registers[reg1 as usize]);
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

    fn visit_mul_upper_s_s(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let a = self.registers[reg0 as usize] as i32 as i64;
        let b = self.registers[reg1 as usize] as i32 as i64;
        let result = ((a * b) >> 32) as u32;
        self.registers[reg2 as usize] = result;
        Ok(())
    }

    fn visit_mul_upper_u_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let a = self.registers[reg0 as usize] as u64;
        let b = self.registers[reg1 as usize] as u64;
        let result = ((a * b) >> 32) as u32;
        self.registers[reg2 as usize] = result;
        Ok(())
    }

    fn visit_mul_upper_s_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let a = self.registers[reg0 as usize] as i32 as i64;
        let b = self.registers[reg1 as usize] as u64;
        let result = ((a * b as i64) >> 32) as u32;
        self.registers[reg2 as usize] = result;
        Ok(())
    }

    fn visit_set_lt_u(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value = if self.registers[reg0 as usize] < self.registers[reg1 as usize] {
            1
        } else {
            0
        };
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_set_lt_s(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let value =
            if (self.registers[reg0 as usize] as i32) < (self.registers[reg1 as usize] as i32) {
                1
            } else {
                0
            };
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_shlo_l(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.registers[reg1 as usize] % 32;
        let value = self.registers[reg0 as usize].wrapping_shl(shift);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_shlo_r(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.registers[reg1 as usize] % 32;
        let value = self.registers[reg0 as usize].wrapping_shr(shift);
        self.registers[reg2 as usize] = value;
        Ok(())
    }

    fn visit_shar_r(&mut self, format: format::RRR) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        let shift = self.registers[reg1 as usize] % 32;
        let value = ((self.registers[reg0 as usize] as i32).wrapping_shr(shift)) as u32;
        self.registers[reg2 as usize] = value;
        Ok(())
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
}
