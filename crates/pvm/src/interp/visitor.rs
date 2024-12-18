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
        let value = self.registers[reg0 as usize] + self.registers[reg1 as usize];
        self.registers[reg2 as usize] = value;
        Ok(())
    }
}
