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

/// only for bench
mod polkavmi;

pub use parser::{Memory, Reader, Register, PAGE_SIZE};
use pvm::{Reason, State};
pub use {
    context::Context,
    result::{Error, Result},
};

/// The interpreter for the polkavm program.
///
/// TODO: maybe use lifetime to save the cost for adpating the
/// invocation interfaces in the future.
#[derive(Default)]
pub struct Interpreter {
    /// The context of the interpreter.
    pub context: Context,

    /// The jump target.
    pub jump: Option<usize>,

    /// The program counter of the interpreter.
    pub pc: usize,

    /// The reason of the exit-execution.
    pub reason: Reason,

    /// The jump table of the interpreter.
    pub table: Vec<u64>,
}

impl Interpreter {
    /// Get the state of the interpreter.
    pub fn state(&self) -> State {
        State {
            pc: self.pc,
            gas: self.context.gas,
            registers: self.context.registers,
            memory: self.context.memory.clone(),
        }
    }

    /// Set the state of the interpreter.
    pub fn set_state(&mut self, state: State) {
        self.pc = state.pc;
        self.context.gas = state.gas;
        self.context.registers = state.registers;
        self.context.memory = state.memory;
    }

    /// Get the output of the interpreter.
    pub fn output(&self) -> Vec<u8> {
        let ptr = self.context.registers[7] as u32;
        let len = self.context.registers[8] as u32;
        self.context.memory.read_bytes(ptr, len).unwrap_or_default()
    }
}
