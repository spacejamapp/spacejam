//! PVM interface implementation

use crate::Interpreter;
use pvm::{Argument, Gas, Invocation, Program, Received};

impl Invocation for Interpreter {
    fn invoke2<X: Argument>(program: &Program, ctx: X, gas: Gas, pc: usize) -> Received<X> {
        Self::invoke(program, ctx, gas, pc).expect("fix me later")
    }
}
