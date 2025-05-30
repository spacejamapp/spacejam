//! PolkaVM program interpreter
//!
//! TODOs:
//!
//! - [ ]: double check the update of program counter
//! - [ ]: double check the jump instruction (what's the exact PC)
//! - [ ]: introduce the sign / unsign transitionss

use crate::{Error, Memory, Register};
use anyhow::Result;
use parser::{Instruction, ProgramBlob, Visitor};
use pvm::Reason;

mod builder;
mod visitor;

/// (Z_A) The alignment factor of the jump table.
pub const JUMP_ALIGNMENT_FACTOR: u32 = 2;

/// The interpreter for the polkavm program.
///
/// TODO: maybe use lifetime to save the cost for adpating the
/// invocation interfaces in the future.
#[derive(Default)]
pub struct Interpreter {
    /// The registers of the interpreter.
    pub registers: [Register; 13],

    /// The gas limit of the interpreter.
    pub gas: u64,

    /// The reason of the exit-execution.
    pub reason: Reason,

    /// The memory of the interpreter.
    pub memory: Memory,

    /// The jump table of the program.
    pub table: Vec<u64>,

    /// The program counter.
    pub pc: usize,

    /// The jump target.
    pub jump: Option<usize>,
}

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
            if let Err(e) = self.step(instr.value) {
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
    fn branch(&mut self, offset: i32, jump: bool) -> crate::Result<()> {
        if jump {
            // TODO:
            // - block checks, need to get access to the reader.
            self.jump = Some((self.pc as i32 + offset) as usize);
        }

        Ok(())
    }

    /// Dynamic jump to the given target.
    fn djump(&mut self, address: u32) -> crate::Result<()> {
        if address == u32::MAX - u16::MAX as u32 {
            return Err(Error::Terminate);
        }

        if address == 0
            || address > self.table.len() as u32 * JUMP_ALIGNMENT_FACTOR
            || address % 2 != 0
        {
            tracing::error!(
                "invalid dynamic jump, address: {}, table len: {}",
                address,
                self.table.len()
            );
            return Err(Error::InvalidDynamicJump);
        }

        let index = address as usize / 2 - 1;
        let Some(target) = self.table.get(index) else {
            tracing::error!(
                "invalid dynamic jump, index: {}, table len: {}",
                index,
                self.table.len()
            );
            return Err(Error::InvalidDynamicJump);
        };

        self.jump = Some(*target as usize);
        Ok(())
    }

    /// Find the jump table index for a given PC address
    /// Returns the index * 2 (with Z_A = 2 alignment) for dynamic jumps
    pub fn pc_to_jump_index(&self, pc: u64) -> Option<u32> {
        for (index, &jump_pc) in self.table.iter().enumerate() {
            if jump_pc == pc {
                // Return index * 2 (Z_A = 2 alignment factor)
                return Some((index + 1) as u32 * 2);
            }
        }
        None
    }

    /// Get the current PC position for return address calculations
    pub fn current_pc(&self) -> u64 {
        self.pc as u64
    }

    /// Get a reference to the jump table for PC to index conversions
    pub fn jump_table(&self) -> &[u64] {
        &self.table
    }
}
