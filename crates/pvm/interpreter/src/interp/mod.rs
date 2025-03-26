//! PolkaVM program interpreter

use crate::{status::Status, Error, Memory, Register};
use anyhow::Result;
use pvm_parser::{program::JumpTable, Instruction, ProgramBlob, Visitor};

mod builder;
mod visitor;

/// (Z_A) The alignment factor of the jump table.
pub const JUMP_ALIGNMENT_FACTOR: u32 = 2;

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

    /// The jump table of the program.
    pub table: JumpTable,

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

        // TODO: do not clone the jump table but reference it.
        self.table = program.jump_table.clone();

        // TODO: update the position of the reader for supporting jumps.
        while !reader.eof() && self.status.is_unknown() {
            let Ok(instr) = reader.read() else {
                tracing::error!("failed to read instruction, position: {}", reader.position);
                return Ok(());
            };

            tracing::trace!("{:08} | {:?}", reader.position, instr.value);
            if let Err(e) = self.step(instr.value) {
                self.status = e.into();
                tracing::error!("error: {e:?}");
                return Ok(());
            }

            // if there is a jump target, update the reader position
            self.pc = reader.position;
            if let Some(pos) = self.jump.take() {
                reader.set_position(pos);
            }
        }

        // If the status is still unknown, we have a trap.
        tracing::debug!("end of program, status: {:?}", self.status);
        if self.status.is_unknown() {
            self.gas -= 1;
            self.status = Status::Panic;
        }

        Ok(())
    }

    /// Step the instruction.
    ///
    /// returns true if the instruction was stepped, false otherwise.
    fn step(&mut self, instr: Instruction) -> crate::Result<()> {
        if self.gas == 0 {
            return Err(Error::OOG);
        }

        self.gas -= 1;
        if let Err(e) = self.visit(instr) {
            self.gas -= e.extra_gas();
            return Err(e);
        }

        Ok(())
    }

    /// Branch to the given target.
    fn branch(&mut self) -> crate::Result<()> {
        Ok(())
    }

    /// Dynamic jump to the given target.
    fn djump(&mut self, address: u32) -> crate::Result<()> {
        if address == u32::MAX - u16::MAX as u32 {
            return Err(Error::Terminate);
        }

        if address == 0
            || address > self.table.len as u32 * JUMP_ALIGNMENT_FACTOR
            || address % 2 != 0
        {
            tracing::error!(
                "invalid dynamic jump, address: {}, table len: {}",
                address,
                self.table.len
            );
            return Err(Error::InvalidDynamicJump);
        }

        let index = address as usize / 2 - 1;
        let Some(target) = self.table.get(index) else {
            tracing::error!(
                "invalid dynamic jump, index: {}, table len: {}",
                index,
                self.table.len
            );
            return Err(Error::InvalidDynamicJump);
        };

        self.jump = Some(target);
        Ok(())
    }
}
