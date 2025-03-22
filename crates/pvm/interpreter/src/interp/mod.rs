//! PolkaVM program interpreter

use crate::{status::Status, Memory, Register};
use anyhow::Result;
use pvm_parser::{Instruction, ProgramBlob, Visitor};

mod builder;
mod visitor;

/// The interpreter for the polkavm program.
#[derive(Default)]
pub struct Interpreter {
    /// The registers of the interpreter.
    pub registers: [Register; 13],

    /// The gas limit of the interpreter.
    pub gas: u32,

    /// The status of the execution.
    pub status: Status,

    /// The memory of the interpreter.
    pub memory: Memory,

    /// The program counter.
    pub pc: usize,

    /// The jump target.
    jump: Option<usize>,
}

impl Interpreter {
    /// Run the program.
    pub fn interp(&mut self, program: impl AsRef<[u8]>) -> Result<()> {
        let program = ProgramBlob::try_from(program.as_ref())?;
        let mut reader = program.instr_reader_at(self.pc);

        // TODO: update the position of the reader for supporting jumps.
        while !reader.eof() && self.status.is_unknown() {
            if self.gas == 0 {
                self.status = Status::OOG;
                return Ok(());
            }

            self.gas -= 1;
            let Ok(instr) = reader.read() else {
                tracing::error!("failed to read instruction, position: {}", reader.position);
                return Ok(());
            };

            tracing::trace!(
                "{:08} | stepped instruction: {:?}",
                reader.position,
                instr.value
            );

            // step the instruction
            if let Err(e) = self.visit(instr.value) {
                self.gas -= 1;
                self.status = e.into();
                break;
            }

            // if there is a jump target, update the reader position
            if let Some(jump) = self.jump.take() {
                if jump > reader.buffer.len() {
                    self.status = Status::Halt;
                    self.pc = 0;
                    return Ok(());
                }

                if jump == 0 {
                    self.status = Status::Panic;
                    self.pc = 0;
                    return Ok(());
                }

                tracing::debug!("jumping to {:08}", jump);
                reader.set_position(jump);
            }

            if self.status.is_trap() {
                break;
            }
            self.pc = reader.position;
        }

        // If the status is still unknown, we have a trap.
        tracing::debug!("end of program, status: {:?}", self.status);
        if self.status.is_unknown() {
            self.gas -= 1;
            if self.visit(Instruction::Trap).is_err() {
                self.status = Status::Panic;
            }
        }

        Ok(())
    }
}
