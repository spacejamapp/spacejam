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
        let is_polkavm = match std::env::var("POLKAVM") {
            Ok(v) => v.to_lowercase() == "true",
            Err(_) => false,
        };

        let now = std::time::Instant::now();
        if is_polkavm {
            let res = crate::polkavmi::invoke(ctx, &code, &args, gas, pc).expect("fix me later");
            println!(
                "Polka VM TIME: {:?}, gas: {}, output: {}",
                now.elapsed(),
                res.gas,
                res.output.len()
            );
            res
        } else {
            let program = program::preimage(code, &args).expect("failed to preimage");
            let res = Interpreter::invoke(&program, ctx, gas, pc).expect("fix me later");
            println!(
                "Space VM TIME: {:?}, gas: {}, output: {}",
                now.elapsed(),
                res.gas,
                res.output.len()
            );
            res
        }
    }
}
