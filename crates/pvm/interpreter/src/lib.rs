//! PVM is a virtual machine for the PVM assembly language.
//!
//! # TODOs:
//!
//! - error handling for index out of bounds.
//! - embed execution result in step outputs.
//! - calculate gas from step outputs.

mod context;
mod interp;
mod invoke;
mod pvmi;
mod result;
mod visitor;

pub use context::Context;
pub use parser::{Memory, Reader, Register, PAGE_SIZE};
use pvm::{Reason, State};
pub use result::{Error, Result};

/// The interpreter for the polkavm program.
///
/// TODO: maybe use lifetime to save the cost for adpating the
/// invocation interfaces in the future.
#[derive(Default)]
pub struct Interpreter {
    /// The gas of the interpreter.
    pub gas: i64,

    /// The jump target.
    pub jump: Option<usize>,

    /// The memory of the interpreter.
    pub memory: parser::Memory,

    /// The program counter of the interpreter.
    pub pc: usize,

    /// The reason of the exit-execution.
    pub reason: Reason,

    /// The registers of the interpreter.
    pub registers: [u64; 13],

    /// The jump table of the interpreter.
    pub table: Vec<u64>,
}

impl Interpreter {
    /// Get the state of the interpreter.
    pub fn state(&self) -> State {
        State {
            pc: self.pc,
            gas: self.gas,
            registers: self.registers,
            memory: self.memory.clone(),
        }
    }

    /// Set the state of the interpreter.
    pub fn set_state(&mut self, state: State) {
        self.pc = state.pc;
        self.gas = state.gas;
        self.registers = state.registers;
        self.memory = state.memory;
    }

    /// Get the output of the interpreter.
    pub fn output(&self) -> Vec<u8> {
        let ptr = self.registers[7] as u32;
        let len = self.registers[8] as u32;
        self.memory.read_bytes(ptr, len).unwrap_or_default()
    }
}
