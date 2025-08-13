//! Formatter for the interpreter.

use crate::{format, visitor, Visitor};
use anyhow::Result;

/// Logger for the interpreter.
pub struct Logger;

impl Visitor for Logger {
    type Error = anyhow::Error;

    fn visit_add_imm_32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "i32 {} = {} + 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );

        Ok(())
    }

    fn visit_add_imm_64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "u64 {} = {} + 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );

        Ok(())
    }

    fn visit_store_ind_u8(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "u8 [{} + 0x{:x}] = {}",
            visitor::fmt(reg1),
            imm0,
            visitor::fmt(reg0),
        );

        Ok(())
    }

    fn visit_store_ind_u16(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "u16 [{} + 0x{:x}] = {}",
            visitor::fmt(reg1),
            imm0,
            visitor::fmt(reg0),
        );

        Ok(())
    }

    fn visit_store_imm_u8(&mut self, format: format::II, _pc: usize) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        tracing::trace!("u8 [0x{:x}] = {}", imm1, imm0);
        Ok(())
    }

    fn visit_store_imm_u16(&mut self, format: format::II, _pc: usize) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        tracing::trace!("u16 [0x{:x}] = {}", imm1, imm0);
        Ok(())
    }

    fn visit_store_imm_u32(&mut self, format: format::II, _pc: usize) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        tracing::trace!("u32 [0x{:x}] = {}", imm1, imm0);
        Ok(())
    }

    fn visit_store_imm_u64(&mut self, format: format::II, _pc: usize) -> Result<()> {
        let format::II { imm0, imm1 } = format;
        tracing::trace!("u64 [0x{:x}] = {}", imm1, imm0);
        Ok(())
    }

    fn visit_store_u8(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("u8 [0x{:x}] = {}", imm0, visitor::fmt(reg0));
        Ok(())
    }

    fn visit_store_u16(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("u16 [0x{:x}] = {}", imm0, visitor::fmt(reg0));
        Ok(())
    }

    fn visit_store_u32(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("u32 [0x{:x}] = {}", imm0, visitor::fmt(reg0));
        Ok(())
    }

    fn visit_store_u64(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("u64 [0x{:x}] = {}", imm0, visitor::fmt(reg0));
        Ok(())
    }

    fn visit_store_imm_ind_u8(&mut self, format: format::RII, _pc: usize) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        tracing::trace!("u8 [{} + 0x{:x}] = {}", visitor::fmt(reg0), imm1, imm0,);
        Ok(())
    }

    fn visit_store_imm_ind_u16(&mut self, format: format::RII, _pc: usize) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        tracing::trace!("u16 [{} + 0x{:x}] = {}", visitor::fmt(reg0), imm1, imm0,);
        Ok(())
    }

    fn visit_store_imm_ind_u32(&mut self, format: format::RII, _pc: usize) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        tracing::trace!("u32 [{} + 0x{:x}] = {}", visitor::fmt(reg0), imm1, imm0,);
        Ok(())
    }

    fn visit_store_imm_ind_u64(&mut self, format: format::RII, _pc: usize) -> Result<()> {
        let format::RII { reg0, imm0, imm1 } = format;
        tracing::trace!("u64 [{} + 0x{:x}] = {}", visitor::fmt(reg0), imm1, imm0,);
        Ok(())
    }

    fn visit_store_ind_u32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "u32 [{} + 0x{:x}] = {}",
            visitor::fmt(reg1),
            imm0,
            visitor::fmt(reg0),
        );

        Ok(())
    }

    fn visit_store_ind_u64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "u64 [{} + 0x{:x}] = {}",
            visitor::fmt(reg1),
            imm0,
            visitor::fmt(reg0),
        );

        Ok(())
    }

    fn visit_jump(&mut self, _format: format::O, _pc: usize) -> Result<()> {
        Ok(())
    }

    fn visit_jump_ind(&mut self, _format: format::RI, _pc: usize) -> Result<()> {
        Ok(())
    }

    fn visit_move_reg(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!("{} = {}", visitor::fmt(reg0), visitor::fmt(reg1));
        Ok(())
    }

    fn visit_load_imm_jump(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("{} = 0x{:x}", visitor::fmt(reg0), imm0,);
        Ok(())
    }

    fn visit_branch_eq_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} == {}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_branch_ge_u(&mut self, format: format::RRO, _pc: usize) -> Result<()> {
        let format::RRO {
            reg0,
            off0: _,
            reg1,
        } = format;
        tracing::trace!(
            "jump ... if {} >=u {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_branch_ge_u_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} >=u 0x{:x}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_branch_lt_s(&mut self, format: format::RRO, _pc: usize) -> Result<()> {
        let format::RRO {
            reg0,
            off0: _,
            reg1,
        } = format;
        tracing::trace!(
            "jump ... if {} <s {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_branch_lt_s_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} <s 0x{:x}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_branch_le_s_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} <=s 0x{:x}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_branch_le_u_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} <=u 0x{:x}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_branch_eq(&mut self, format: format::RRO, _pc: usize) -> Result<()> {
        let format::RRO {
            reg0,
            off0: _,
            reg1,
        } = format;
        tracing::trace!(
            "jump ... if {} == {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_branch_ge_s(&mut self, format: format::RRO, _pc: usize) -> Result<()> {
        let format::RRO {
            reg0,
            off0: _,
            reg1,
        } = format;
        tracing::trace!(
            "jump ... if {} >=s {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_branch_ge_s_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} >=s 0x{:x}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_branch_ne(&mut self, format: format::RRO, _pc: usize) -> Result<()> {
        let format::RRO {
            reg0,
            off0: _,
            reg1,
        } = format;
        tracing::trace!(
            "jump ... if {} != {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_branch_gt_s_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} >s 0x{:x}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_branch_gt_u_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} >u 0x{:x}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_branch_ne_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} != 0x{:x}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_shlo_l_imm_64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} << 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );

        Ok(())
    }

    fn visit_shlo_l_imm_alt_64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} << {}",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1),
        );

        Ok(())
    }

    fn visit_shlo_r_imm_64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} >> 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );

        Ok(())
    }

    fn visit_shar_r_imm_64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} >> 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );

        Ok(())
    }

    fn visit_shar_r_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} >> {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shar_r_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} >> {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shar_r_imm_32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} >> 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );
        Ok(())
    }

    fn visit_shar_r_imm_alt_32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} >> {}",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shar_r_imm_alt_64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} >> {}",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shlo_l_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} << {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shlo_l_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} << {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shlo_l_imm_32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} << 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );
        Ok(())
    }

    fn visit_shlo_l_imm_alt_32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} << {}",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shlo_r_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} >> {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shlo_r_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} >> {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shlo_r_imm_32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} >> 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );
        Ok(())
    }

    fn visit_shlo_r_imm_alt_32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} >> {}",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_shlo_r_imm_alt_64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} >> {}",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1),
        );
        Ok(())
    }

    fn visit_load_imm(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("{} = 0x{:x}", visitor::fmt(reg0), imm0,);
        Ok(())
    }

    fn visit_load_ind_u8(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "u8 {} = [{} + 0x{:x}]",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );

        Ok(())
    }

    fn visit_load_ind_u16(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "u16 {} = [{} + 0x{:x}]",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );

        Ok(())
    }

    fn visit_load_ind_u32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "u32 {} = [{} + 0x{:x}]",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );

        Ok(())
    }

    fn visit_load_ind_u64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "u64 {} = [{} + 0x{:x}]",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0,
        );

        Ok(())
    }

    fn visit_branch_lt_u(&mut self, format: format::RRO, _pc: usize) -> Result<()> {
        let format::RRO {
            reg0,
            off0: _,
            reg1,
        } = format;
        tracing::trace!(
            "jump ... if {} <u {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_branch_lt_u_imm(&mut self, format: format::RIO, _pc: usize) -> Result<()> {
        let format::RIO {
            reg0,
            off0: _,
            imm0,
        } = format;
        tracing::trace!("jump ... if {} <u 0x{:x}", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_set_lt_u(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} <u {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );

        Ok(())
    }

    fn visit_add_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} + {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1),
        );

        Ok(())
    }

    fn visit_add_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} + {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1),
        );

        Ok(())
    }

    fn visit_sub_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} - {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );

        Ok(())
    }

    fn visit_sub_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} - {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );

        Ok(())
    }

    fn visit_cmov_nz(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} if {} != 0",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );

        Ok(())
    }

    fn visit_fallthrough(&mut self, _pc: usize) -> Result<()> {
        tracing::trace!("unresolved fallthrough");
        Ok(())
    }

    fn visit_sign_extend_8(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!("{} = sext.b {}", visitor::fmt(reg0), visitor::fmt(reg1));
        Ok(())
    }

    fn visit_sign_extend_16(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!("{} = sext.b {}", visitor::fmt(reg0), visitor::fmt(reg1));
        Ok(())
    }

    fn visit_xor(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} ^ {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );

        Ok(())
    }

    fn visit_xor_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} ^ 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );

        Ok(())
    }

    fn visit_xnor(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} ^^ {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );

        Ok(())
    }

    fn visit_max(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = max ({}, {})",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );

        Ok(())
    }

    fn visit_min(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = min ({}, {})",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );

        Ok(())
    }

    fn visit_and(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} & {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_and_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} & 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_and_inv(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} & ~{}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_cmov_iz(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} if {} == 0",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_cmov_iz_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} if {} == 0",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_cmov_nz_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} if {} != 0",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_count_set_bits_32(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!(
            "{} = popcount32({})",
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_count_set_bits_64(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!(
            "{} = popcount64({})",
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_div_s_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} /s {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_div_s_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} /s {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_div_u_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} /u {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_div_u_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} /u {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_ecalli(&mut self, format: format::I, _pc: usize) -> Result<()> {
        let format::I { imm0 } = format;
        tracing::trace!("ecall 0x{:x}", imm0);
        Ok(())
    }

    fn visit_leading_zero_bits_32(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!("{} = clz32({})", visitor::fmt(reg0), visitor::fmt(reg1));
        Ok(())
    }

    fn visit_leading_zero_bits_64(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!("{} = clz64({})", visitor::fmt(reg0), visitor::fmt(reg1));
        Ok(())
    }

    fn visit_load_i16(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("i16 {} = [0x{:x}]", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_load_i32(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("i32 {} = [0x{:x}]", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_load_i8(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("i8 {} = [0x{:x}]", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_load_imm_64(&mut self, format: format::REI, _pc: usize) -> Result<()> {
        let format::REI { reg0, eimm0 } = format;
        tracing::trace!("{} = 0x{:x}", visitor::fmt(reg0), eimm0);
        Ok(())
    }

    fn visit_load_imm_jump_ind(&mut self, format: format::RRII, _pc: usize) -> Result<()> {
        let format::RRII {
            reg0,
            reg1,
            imm0,
            imm1: _,
        } = format;
        tracing::trace!(
            "{} = 0x{:x}, jump [{}]",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_load_ind_i16(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "i16 {} = [{} + 0x{:x}]",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_load_ind_i32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "i32 {} = [{} + 0x{:x}]",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_load_ind_i8(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "i8 {} = [{} + 0x{:x}]",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_load_u16(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("u16 {} = [0x{:x}]", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_load_u32(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("u32 {} = [0x{:x}]", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_load_u64(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("u64 {} = [0x{:x}]", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_load_u8(&mut self, format: format::RI, _pc: usize) -> Result<()> {
        let format::RI { reg0, imm0 } = format;
        tracing::trace!("u8 {} = [0x{:x}]", visitor::fmt(reg0), imm0);
        Ok(())
    }

    fn visit_max_u(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = max_u ({}, {})",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );
        Ok(())
    }

    fn visit_min_u(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = min_u ({}, {})",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );
        Ok(())
    }

    fn visit_mul_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} * {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_mul_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} * {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_mul_imm_32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} * 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_mul_imm_64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} * 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_mul_upper_s_s(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = mulh_ss ({}, {})",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_mul_upper_s_u(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = mulh_su ({}, {})",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_mul_upper_u_u(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = mulh_uu ({}, {})",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_neg_add_imm_32(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} - {}",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_neg_add_imm_64(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = 0x{:x} - {}",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_or(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} | {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_or_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} | 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_or_inv(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} | ~{}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_rem_s_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} %s {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_rem_s_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} %s {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_rem_u_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} %u {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_rem_u_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} %u {}",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_reverse_bytes(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!("{} = bswap({})", visitor::fmt(reg0), visitor::fmt(reg1));
        Ok(())
    }

    fn visit_rot_l_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = rotl32({}, {})",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_rot_l_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = rotl64({}, {})",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_rot_r_32(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = rotr32({}, {})",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_rot_r_32_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = rotr32({}, 0x{:x})",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_rot_r_32_imm_alt(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = rotr32(0x{:x}, {})",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_rot_r_64(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = rotr64({}, {})",
            visitor::fmt(reg2),
            visitor::fmt(reg0),
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_rot_r_64_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = rotr64({}, 0x{:x})",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_rot_r_64_imm_alt(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = rotr64(0x{:x}, {})",
            visitor::fmt(reg0),
            imm0,
            visitor::fmt(reg1)
        );
        Ok(())
    }

    fn visit_sbrk(&mut self, _format: format::RR, _pc: usize) -> Result<()> {
        Ok(())
    }

    fn visit_set_gt_s_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} >s 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_set_gt_u_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} >u 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_set_lt_s(&mut self, format: format::RRR, _pc: usize) -> Result<()> {
        let format::RRR { reg0, reg1, reg2 } = format;
        tracing::trace!(
            "{} = {} <s {}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            visitor::fmt(reg2)
        );
        Ok(())
    }

    fn visit_set_lt_s_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} <s 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_set_lt_u_imm(&mut self, format: format::RRI, _pc: usize) -> Result<()> {
        let format::RRI { reg0, reg1, imm0 } = format;
        tracing::trace!(
            "{} = {} <u 0x{:x}",
            visitor::fmt(reg0),
            visitor::fmt(reg1),
            imm0
        );
        Ok(())
    }

    fn visit_trailing_zero_bits_32(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!("{} = ctz32({})", visitor::fmt(reg0), visitor::fmt(reg1));
        Ok(())
    }

    fn visit_trailing_zero_bits_64(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!("{} = ctz64({})", visitor::fmt(reg0), visitor::fmt(reg1));
        Ok(())
    }

    fn visit_trap(&mut self, _pc: usize) -> Result<()> {
        tracing::trace!("trap");
        Ok(())
    }

    fn visit_zero_extend_16(&mut self, format: format::RR, _pc: usize) -> Result<()> {
        let format::RR { reg0, reg1 } = format;
        tracing::trace!("{} = zext.h {}", visitor::fmt(reg0), visitor::fmt(reg1));
        Ok(())
    }
}
