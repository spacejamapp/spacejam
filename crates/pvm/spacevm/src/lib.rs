//! Jastime - JAM virtual machine

use pvm::{
    parser,
    score::{Gas, OpaqueHash},
    Argument, Invocation, Invoked, State,
};
use pvmc::Module;
pub use pvmc::{Artifact, Compiler};
pub use pvmi::Interpreter;
use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock, RwLock},
};

/// Locks for the Jastime compilation
pub static SPACEVM_MODULES: LazyLock<RwLock<BTreeMap<OpaqueHash, Arc<Module>>>> =
    LazyLock::new(|| RwLock::new(BTreeMap::new()));

/// Locks for the Jastime compilation
pub static SPACEVM_LOCKS: LazyLock<RwLock<BTreeMap<OpaqueHash, ()>>> =
    LazyLock::new(|| RwLock::new(BTreeMap::new()));

/// Jastime - JAM virtual machine
pub struct SpaceVM;

impl Invocation for SpaceVM {
    fn invoke2<X: Argument>(
        mut ctx: X,
        hash: OpaqueHash,
        code: Vec<u8>,
        args: Vec<u8>,
        gas: Gas,
        pc: usize,
    ) -> Invoked<X> {
        if let Ok(None) = SPACEVM_LOCKS.read().map(|lock| lock.get(&hash).cloned()) {
            if let Ok(Some(module)) = SPACEVM_MODULES.read().map(|lock| lock.get(&hash).cloned()) {
                let mut context = pvm::Context {
                    registers: module.registers,
                    gas: gas as i64,
                    memory: module.memory.clone(),
                    ctx: &mut ctx,
                };

                let reason = module
                    .execute(&mut context, pc as u64)
                    .expect("fix me later");
                return Invoked {
                    gas: gas - (context.gas.max(0) as u64),
                    output: Default::default(),
                    reason,
                    state: State {
                        pc: 0,
                        gas: context.gas,
                        registers: context.registers,
                        memory: Default::default(),
                    },
                    data: ctx,
                };
            }
        }

        // lock the compilation
        {
            let code = code.clone();
            let args = args.clone();
            tokio::spawn(async move {
                if let Ok(mut locks) = SPACEVM_LOCKS.write() {
                    locks.insert(hash, ());
                }

                match Compiler::host::<()>()
                    .expect("fix me later")
                    .compile_with_cache(
                        &parser::program::preimage(code, &args).expect("failed to preimage"),
                        Some(hash),
                    ) {
                    Ok(module) => {
                        if let Ok(mut locks) = SPACEVM_MODULES.write() {
                            locks.insert(hash, Arc::new(module));
                        }
                    }
                    Err(err) => {
                        tracing::warn!("failed to compile program: {:?}", err);
                    }
                }

                if let Ok(mut locks) = SPACEVM_LOCKS.write() {
                    locks.remove(&hash);
                }
            });
        }

        // fallback to the interpreter
        Interpreter::invoke2(ctx, hash, code, args, gas, pc)
    }
}
