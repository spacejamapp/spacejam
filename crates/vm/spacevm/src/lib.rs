//! Jastime - JAM virtual machine

use anyhow::Result;
use lru::LruCache;
pub use pvm;
use pvm::{
    Argument, Invocation, Invoked, Pvm, State, parser,
    score::{Gas, OpaqueHash},
};
pub use pvmc::{Artifact, Compiler, Memory, ModuleLike, SPACEJAM_CACHE_DIR, numa};
pub use pvmi::Interpreter;
use std::{
    collections::BTreeSet,
    num::NonZeroUsize,
    sync::{Arc, LazyLock, Mutex, RwLock},
    thread,
};

/// Default max modules kept in memory.
const DEFAULT_MODULE_CACHE: usize = 8;

/// Effective cache size.
fn module_cache_size() -> NonZeroUsize {
    std::env::var("SPACEJAM_MODULE_CACHE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .and_then(NonZeroUsize::new)
        .unwrap_or_else(|| NonZeroUsize::new(DEFAULT_MODULE_CACHE).expect("nonzero default"))
}

/// Cached modules (LRU). Uses Mutex because LruCache::get requires &mut for LRU tracking.
pub static SPACEVM_MODULES: LazyLock<Mutex<LruCache<OpaqueHash, Arc<pvmc::Module>>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(module_cache_size())));

/// Locks for the Jastime compilation
pub static SPACEVM_LOCKS: LazyLock<RwLock<BTreeSet<OpaqueHash>>> =
    LazyLock::new(|| RwLock::new(BTreeSet::new()));

/// SpaceVM - JAM virtual machine
pub struct SpaceVM;

impl Pvm for SpaceVM {
    fn install<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + Send,
        R: Send,
    {
        pvmc::numa::pool().install(f)
    }
}

impl Invocation for SpaceVM {
    fn invoke2<X: Argument>(
        mut ctx: X,
        hash: OpaqueHash,
        code: Vec<u8>,
        args: Vec<u8>,
        gas: Gas,
        pc: usize,
    ) -> Invoked<X> {
        let module = if let Ok(true) = SPACEVM_LOCKS.read().map(|lock| !lock.contains(&hash))
            && let Ok(Some(module)) = SPACEVM_MODULES
                .lock()
                .map(|mut cache| cache.get(&hash).cloned())
        {
            Some(module)
        } else if let Ok(module) = <pvmc::Module as ModuleLike>::new::<X>()
            && let Ok(program) = parser::program::preimage(code.clone(), &args)
            && let Ok(Some(module)) = ModuleLike::try_load(module, &program)
        {
            let arc = Arc::new(module);
            if let Ok(mut cache) = SPACEVM_MODULES.lock() {
                cache.put(hash, arc.clone());
            }
            Some(arc)
        } else {
            None
        };

        if let Some(module) = module {
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

            let output = {
                let ptr = context.registers[7] as u32;
                let len = context.registers[8] as u32;
                let mut buf = vec![0u8; len as usize];
                match pvmc::trap::with(|| {
                    buf.copy_from_slice(context.memory.read_bytes(ptr, len).as_ref());
                }) {
                    Ok(()) => buf,
                    Err(_) => Vec::new(),
                }
            };
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
                        tracing::debug!("failed to compile program: {e:?}");
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
            if memcache && let Ok(mut cache) = SPACEVM_MODULES.lock() {
                cache.put(hash, Arc::new(module));
            }
        }
        Err(err) => {
            tracing::debug!("failed to compile program: {:?}", err);
        }
    }

    if let Ok(mut locks) = SPACEVM_LOCKS.write() {
        locks.remove(&hash);
    }
    Ok(())
}
