//! PolkaVM program interpreter
//!
//! TODOs:
//!
//! - [ ]: double check the update of program counter
//! - [ ]: double check the jump instruction (what's the exact PC)
//! - [ ]: introduce the sign / unsign transitionss

use crate::{status::Status, Error, Memory, Register};
use anyhow::Result;
use pvm::{Gas, Invocation, Reason, Stepped};
use pvm_parser::{
    reader::{InstructionReader, Reader},
    Instruction, ProgramBlob, Visitor,
};

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
    pub gas: u64,

    /// The status of the execution.
    pub status: Status,

    /// The memory of the interpreter.
    pub memory: Memory,

    /// The jump table of the program.
    pub table: Vec<u64>,

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

            // stepping the instruction.
            tracing::trace!("0x{:06x} | {}", self.pc, instr.value);
            if let Err(e) = self.step(instr.value) {
                self.status = e.into();
                return Ok(());
            }

            // update the program counter on stepping successfully.
            self.pc = reader.position;

            // if there is a jump target, update the reader position
            if let Some(pos) = self.jump.take() {
                reader.set_position(pos);
                self.pc = pos;
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
}

impl Invocation for Interpreter {
    type Memory = Memory;

    /// Step the instruction.
    fn step(
        // (c) The instruction data
        instructions: &[u8],
        // (k) The bitmap of the instruction data
        bitmask: &[u8],
        // (j) The jump table
        jump: &[u64],
        // (ı) The current program counter
        pc: u64,
        // (ϱ) The gas
        gas: Gas,
        // (ω) The registers
        registers: [u64; 13],
        // (µ) The memory
        memory: Memory,
    ) -> Stepped<Memory, ()> {
        let pc = pc as usize;
        let mut pvmi = Interpreter::default()
            .gas(gas)
            .registers(registers)
            .memory(memory.into())
            .pc(pc);
        pvmi.table = jump.to_vec();
        let mut state = pvm::State {
            memory: pvmi.memory.clone().into(),
            registers,
            gas: gas as i64,
            pc: pc as u64,
        };

        // check if the program counter is out of bounds
        if pc >= instructions.len() {
            state.gas -= 1;
            return Stepped::new(Reason::Panic("end of program".to_string()), state);
        }

        // create the instruction reader
        let mut reader = InstructionReader {
            bitmask,
            reader: Reader::new(&instructions, pc),
        }
        .with_position(pc);

        // get the opcode
        let instr = match reader.read() {
            Ok(instr) => instr,
            Err(e) => {
                tracing::error!("invalid instruction: {}", e);
                return Stepped::new(Reason::Panic(e.to_string()), state);
            }
        };

        // step the instruction
        tracing::trace!("0x{:06x} | {}", pc, instr.value);
        let stepped = pvmi.visit(instr.value);

        // update the state
        state.gas -= 1;
        state.registers = pvmi.registers;
        state.memory = pvmi.memory.clone();

        let reason = if let Err(e) = stepped {
            state.gas -= e.extra_gas() as i64;
            match e {
                crate::Error::OOG => Reason::OOG,
                crate::Error::Terminate => Reason::Halt,
                crate::Error::Trap(_) => Reason::Panic("trap".to_string()),
                crate::Error::InvalidDynamicJump => {
                    Reason::Panic("invalid dynamic jump".to_string())
                }
                crate::Error::MemoryInaccessible(page) => Reason::Fault(page),
                crate::Error::MemoryImmutable(page) => Reason::Fault(page),
            }
        } else {
            if let Some(pos) = pvmi.jump.take() {
                state.pc = pos as u64;
            } else {
                state.pc = reader.position as u64;
            }
            Reason::Continue
        };

        Stepped::new(reason, state)
    }
}
