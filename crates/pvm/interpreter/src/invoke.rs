//! Invocation APIs of the interpreter

use crate::Interpreter;
use anyhow::Result;
use pvm::{Argument, Gas, Program};

impl Interpreter {
    /// Interp a program without any context
    pub fn interp(program: &Program, gas: Gas, pc: usize) -> Result<()> {
        Ok(())
    }

    /// Invoke a program with the given context
    pub fn invoke<X: Argument>(program: &Program, ctx: X, gas: Gas, pc: usize) -> Result<()> {
        Ok(())
    }
}
