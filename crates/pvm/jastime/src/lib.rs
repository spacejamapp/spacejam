//! Jastime - JAM virtual machine

use pvm::{
    parser,
    score::{Gas, OpaqueHash},
    Argument, Invocation, Invoked,
};
pub use pvmc::{Artifact, Compiler};
pub use pvmi::Interpreter;

/// Jastime - JAM virtual machine
pub struct Jastime;

impl Jastime {
    /// Compile a program
    pub fn compile<X: Argument>(hash: OpaqueHash, code: Vec<u8>, args: Vec<u8>) {
        let program = parser::program::preimage(code, &args).expect("failed to preimage");
        Compiler::new()
            .expect("fix me later")
            .compile_with_cache::<X>(&program, Some(hash))
            .expect("fix me later");
    }
}

impl Invocation for Jastime {
    fn invoke2<X: Argument>(
        ctx: X,
        hash: OpaqueHash,
        code: Vec<u8>,
        args: Vec<u8>,
        gas: Gas,
        pc: usize,
    ) -> Invoked<X> {
        let artifacts = Artifact::new().expect("failed to create artifact");
        if artifacts.hits(hash) {
            return Compiler::invoke2(ctx, hash, code, args, gas, pc);
        }

        {
            let code = code.clone();
            let args = args.clone();
            tokio::task::spawn_blocking(move || {
                Jastime::compile::<X>(hash, code, args);
            });
        }

        Interpreter::invoke2(ctx, hash, code, args, gas, pc)
    }
}
