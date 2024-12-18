//! PolkaVM program interpreter

use crate::{status::Status, Memory};
use anyhow::Result;
use parser::{reader::Offset, Instruction, ProgramBlob, Visitor};

mod builder;
mod visitor;

/// The interpreter for the polkavm program.
#[derive(Default)]
pub struct Interpreter {
    /// The registers of the interpreter.
    pub registers: [u32; 13],

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
            let instr = reader.read()?;
            self.step(instr)?;

            // if there is a jump target, update the reader position
            if let Some(jump) = self.jump.take() {
                reader.set_position(jump);
            }

            self.pc = reader.position;
        }

        // If the status is still unknown, we have a trap.
        if self.status.is_unknown() {
            self.step(Offset {
                range: self.pc..self.pc,
                value: Instruction::Trap,
            })?;
        }

        Ok(())
    }

    /// Execute a single instruction.
    pub fn step(&mut self, instr: Offset<Instruction>) -> Result<()> {
        tracing::debug!("stepping instruction: {:?}", instr.value);
        if self.gas == 0 {
            self.status = Status::OutOfGas;
            return Ok(());
        }

        self.visit(instr.value)?;
        self.gas -= 1;
        Ok(())
    }
}

#[test]
fn test_add() {
    let mut interpreter = Interpreter::default()
        .registers([0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0])
        .gas(10000);

    interpreter
        .interp([0, 0, 3, 8, 135, 9, 1])
        .expect("interp failed");
    assert_eq!(interpreter.status, Status::Trap);
    assert_eq!(
        interpreter.registers,
        [0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 0, 0, 0]
    );
}
