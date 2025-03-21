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
            let Ok(instr) = reader.read() else {
                tracing::error!("failed to read instruction, position: {}", reader.position);
                self.status = Status::Trap;
                return Ok(());
            };

            tracing::trace!(
                "{:08} | stepped instruction: {:?}",
                reader.position,
                instr.value
            );

            // if there is a jump target, update the reader position
            self.step(instr)?;
            if let Some(jump) = self.jump.take() {
                if jump > reader.buffer.len() {
                    self.status = Status::Halt;
                    self.pc = 0;
                    return Ok(());
                }

                if jump == 0 {
                    self.status = Status::Trap;
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
            self.step(Offset {
                range: self.pc..self.pc,
                value: Instruction::Trap,
            })?;
        }

        Ok(())
    }

    /// Execute a single instruction.
    pub fn step(&mut self, instr: Offset<Instruction>) -> Result<()> {
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
        .interp([0, 0, 3, 0xbe, 135, 9, 1])
        .expect("interp failed");
    assert_eq!(interpreter.status, Status::Trap);
    assert_eq!(
        interpreter.registers,
        [0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 0, 0, 0]
    );
}
