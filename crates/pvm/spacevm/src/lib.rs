//! Jastime - JAM virtual machine

use anyhow::Result;
pub use pvm;
use pvm::{
    Argument, Invocation, Invoked, State, parser,
    score::{Gas, OpaqueHash},
};
pub use pvmc::{Artifact, Compiler, Memory, Module};
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
        if let Ok(None) = SPACEVM_LOCKS.read().map(|lock| lock.get(&hash).cloned())
            && let Ok(Some(module)) = SPACEVM_MODULES.read().map(|lock| lock.get(&hash).cloned())
        {
            let program = parser::program::preimage(code, &args).expect("failed to preimage");
            let mut context = pvm::Context {
                registers: program.registers,
                gas: gas as i64,
                memory: Memory::new(&program.memory).expect("failed to create memory"),
                ctx: &mut ctx,
            };

            let reason = module
                .execute(&mut context, pc as u64)
                .expect("fix me later");

            // TODO: find a solution to do this without the trap handler
            let output = pvmc::trap::with(|| context.acc_output()).unwrap_or_default();
            return Invoked {
                gas: gas - (context.gas.max(0) as u64),
                output,
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

        // lock the compilation
        {
            if let Ok(locks) = SPACEVM_LOCKS.read()
                && !locks.contains_key(&hash)
            {
                let code = code.clone();
                let args = args.clone();
                tokio::task::spawn_blocking(move || self::compile::<X>(code, args, hash, true));
            }
        }

        // fallback to the interpreter
        Interpreter::invoke2(ctx, hash, code, args, gas, pc)
    }
}

/// Compile a program
pub fn compile<X: Argument>(
    code: Vec<u8>,
    args: Vec<u8>,
    hash: OpaqueHash,
    memcache: bool,
) -> Result<()> {
    if let Ok(mut locks) = SPACEVM_LOCKS.write() {
        locks.insert(hash, ());
    }

    match Compiler::host::<X>()
        .expect("fix me later")
        .compile_with_cache(
            &parser::program::preimage(code, &args).expect("failed to preimage"),
            Some(hash),
        ) {
        Ok(module) => {
            if let Ok(mut locks) = SPACEVM_MODULES.write()
                && memcache
            {
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
    Ok(())
}
