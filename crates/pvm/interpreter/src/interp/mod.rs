//! PolkaVM program interpreter
//!
//! TODOs:
//!
//! - [ ]: double check the update of program counter
//! - [ ]: double check the jump instruction (what's the exact PC)
//! - [ ]: introduce the sign / unsign transitionss

use crate::{Error, Memory, Register};
use pvm::{Accounts, Reason};
use std::marker::PhantomData;

mod builder;
mod legacy;
mod register;
mod visitor;

/// (Z_A) The alignment factor of the jump table.
pub const JUMP_ALIGNMENT_FACTOR: u32 = 2;

/// The interpreter for the polkavm program.
///
/// TODO: maybe use lifetime to save the cost for adpating the
/// invocation interfaces in the future.
pub struct Interpreter<R: Accounts> {
    /// The registers of the interpreter.
    /// ra = [0]
    /// sp = [1]
    ///  s = [5, 6]
    ///  a = [7, 8, 9, 10, 11]
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

    _acc: PhantomData<R>,
}

impl<R: Accounts> Interpreter<R> {
    /// Branch to the given target.
    fn branch(&mut self, offset: i32, jump: bool) -> crate::Result<()> {
        if jump {
            self.jump = Some((self.pc as i32 + offset) as usize);
        }

        Ok(())
    }

    /// Burn the gas.
    pub fn burn(&mut self, gas: u64) -> crate::Result<()> {
        if self.gas < gas {
            return Err(Error::OOG);
        }

        self.gas = self.gas.saturating_sub(gas);
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

        // tracing::trace!("jumping to dynamic index={index} address: {target}");
        self.jump = Some(*target as usize);
        Ok(())
    }
}
