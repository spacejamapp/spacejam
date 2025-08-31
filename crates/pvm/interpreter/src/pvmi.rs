//! PVM interface implementation

use crate::Interpreter;
use parser::program;
use pvm::{
    score::{Gas, OpaqueHash},
    Argument, Invocation, Invoked,
};

impl Invocation for Interpreter {
    fn invoke2<X: Argument>(
        ctx: X,
        _hash: OpaqueHash,
        code: Vec<u8>,
        args: Vec<u8>,
        gas: Gas,
        pc: usize,
    ) -> Invoked<X> {
        let program = program::preimage(code, &args).expect("failed to preimage");
        Self::invoke(&program, ctx, gas, pc).expect("fix me later")
    }
}
