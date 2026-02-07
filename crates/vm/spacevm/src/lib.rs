//! Jastime - JAM virtual machine

use anyhow::Result;
use lru::LruCache;
pub use pvm;
use pvm::{
    Argument, Invocation, Invoked, State, parser,
    score::{Gas, OpaqueHash},
};
pub use pvmc::{Artifact, Compiler, Memory, ModuleLike, SPACEJAM_CACHE_DIR};
pub use pvmi::Interpreter;
use std::{
    collections::BTreeSet,
    num::NonZeroUsize,
    sync::{Arc, LazyLock, Mutex, RwLock},
    thread,
};

/// Maximum number of modules to keep in memory. Evicted modules remain on disk
/// and can be reloaded when needed.
const MAX_CACHED_MODULES: usize = 3;

/// Cached modules (LRU). Uses Mutex because LruCache::get requires &mut for LRU tracking.
pub static SPACEVM_MODULES: LazyLock<Mutex<LruCache<OpaqueHash, Arc<pvmc::Module>>>> =
    LazyLock::new(|| {
        Mutex::new(LruCache::new(
            NonZeroUsize::new(MAX_CACHED_MODULES).expect("MAX_CACHED_MODULES must be non-zero"),
        ))
    });

/// Locks for the Jastime compilation
pub static SPACEVM_LOCKS: LazyLock<RwLock<BTreeSet<OpaqueHash>>> =
    LazyLock::new(|| RwLock::new(BTreeSet::new()));

/// SpaceVM - JAM virtual machine
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
        if let Ok(true) = SPACEVM_LOCKS.read().map(|lock| !lock.contains(&hash))
            && let Ok(Some(module)) = SPACEVM_MODULES
                .lock()
                .map(|mut cache| cache.get(&hash).cloned())
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
                && !locks.contains(&hash)
            {
                let code = code.clone();
                let args = args.clone();
                thread::spawn(move || {
                    if let Err(e) = self::compile::<X>(code, args, hash, true) {
                        tracing::warn!("failed to compile program: {e:?}");
                    }
                });
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
        locks.insert(hash);
    }

    match <pvmc::Module as ModuleLike>::new::<X>()?
        .compile(&parser::program::preimage(code, &args).expect("failed to preimage"))
    {
        Ok(module) => {
            if memcache {
                if let Ok(mut cache) = SPACEVM_MODULES.lock() {
                    cache.put(hash, Arc::new(module));
                }
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
