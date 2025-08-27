//! PVM interface implementation

use crate::Interpreter;
use pvm::{score::Gas, Argument, Invocation, Invoked, Program};

impl Invocation for Interpreter {
    fn invoke2<X: Argument>(program: &Program, ctx: X, gas: Gas, pc: usize) -> Invoked<X> {
        Self::invoke(program, ctx, gas, pc).expect("fix me later")
    }
}
