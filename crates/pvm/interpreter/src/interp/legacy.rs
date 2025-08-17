//! Legacy interpreter interfaces.

use crate::{Error, Interpreter};
use anyhow::Result;
use parser::{reader::Offset, Instruction, ProgramBlob, Visitor};
use pvm::Reason;

impl Interpreter {
    /// Run the program.
    pub fn interp(&mut self, program: impl AsRef<[u8]>) -> Result<()> {
        let program = ProgramBlob::try_from(program.as_ref())?;
        let mut reader = program.reader().with_position(self.pc);

        // stepping the instructions
        self.table = program.jump_table.clone();
        while !reader.eof() && self.reason.is_continue() {
            let Ok(instr) = reader.read() else {
                tracing::error!("failed to read instruction, position: {}", reader.position);
                return Ok(());
            };

            // stepping the instruction.
            tracing::trace!("0x{:06x} | {}", self.pc, instr.value);
            if let Err(e) = self.step(instr) {
                self.reason = e.into();
                return Ok(());
            }

            // update the program counter
            self.pc = reader.position;
            if let Some(pos) = self.jump.take() {
                reader.set_position(pos);
                self.pc = pos;
            }
        }

        // If the reason is still unknown, we have a trap.
        if self.reason.is_continue() {
            self.gas -= 1;
            self.reason = Reason::Panic("end of program".into());
        }

        Ok(())
    }

    /// Step the instruction.
    ///
    /// returns true if the instruction was stepped, false otherwise.
    fn step(&mut self, instr: Offset<Instruction>) -> crate::Result<()> {
        if self.gas == 0 {
            return Err(Error::OOG);
        }

        self.gas -= 1;
        if let Err(e) = self.visit(instr.value, &instr.range) {
            self.gas -= e.extra_gas();
            return Err(e);
        }

        Ok(())
    }
}
