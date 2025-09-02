//! Jastime - JAM virtual machine

use pvm::{
    parser,
    score::{Gas, OpaqueHash},
    Argument, Invocation, Invoked,
};
pub use pvmc::{Artifact, Compiler};
pub use pvmi::Interpreter;
use std::{collections::HashSet, sync::LazyLock};
use tokio::sync::RwLock;

/// Locks for the Jastime compilation
pub static JASTIME_LOCKS: LazyLock<RwLock<HashSet<OpaqueHash>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

/// Jastime - JAM virtual machine
pub struct Jastime;

impl Invocation for Jastime {
    fn invoke2<X: Argument>(
        ctx: X,
        hash: OpaqueHash,
        code: Vec<u8>,
        args: Vec<u8>,
        gas: Gas,
        pc: usize,
    ) -> Invoked<X> {
        let artifact = Artifact::new().expect("failed to create artifact");
        if artifact.hits(hash) {
            return Compiler::invoke2(ctx, hash, code, args, gas, pc);
        }

        // lock the compilation
        {
            let code = code.clone();
            let args = args.clone();
            tokio::spawn(async move {
                if JASTIME_LOCKS.read().await.contains(&hash) {
                    return;
                }

                JASTIME_LOCKS.write().await.insert(hash);
                let _ = Compiler.compile_with_cache::<()>(
                    &parser::program::preimage(code, &args).expect("failed to preimage"),
                    Some(hash),
                );

                JASTIME_LOCKS.write().await.remove(&hash);
            });
        }

        // fallback to the interpreter
        Interpreter::invoke2(ctx, hash, code, args, gas, pc)
    }
}
