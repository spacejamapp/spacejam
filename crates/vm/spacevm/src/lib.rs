//! Jastime - JAM virtual machine

use anyhow::Result;
pub use pvm;
use pvm::{
    Argument, Cache, Invocation, Invoked, Pvm, State, parser,
    score::{Gas, OpaqueHash},
};
pub use pvmc::{Artifact, Compiler, Memory, ModuleLike, SPACEJAM_CACHE_DIR};
pub use pvmi::Interpreter;
use std::sync::{Arc, LazyLock};

/// Cached AOT modules.
pub static SPACEVM_MODULES: LazyLock<Cache<pvmc::Module>> = LazyLock::new(Default::default);

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
        let module = if let Some(module) = SPACEVM_MODULES.get(&hash) {
            Some(module)
        } else if let Ok(module) = <pvmc::Module as ModuleLike>::new::<X>()
            && let Ok(program) = parser::program::preimage(code.clone(), &args)
            && let Ok(Some(module)) = ModuleLike::try_load(module, &program)
        {
            let arc = Arc::new(module);
            SPACEVM_MODULES.put(hash, arc.clone());
            Some(arc)
        } else {
            None
        };

        if let Some(module) = module {
            let program = parser::program::preimage(code, &args).expect("failed to preimage");
            let mut context = pvm::Context {
                registers: program.registers,
                gas: gas as i64,
                memory: Memory::new(hash, &program.memory).expect("failed to create memory"),
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

        // Kick off background AOT compile.
        {
            let code = code.clone();
            let args = args.clone();
            rayon::spawn(move || {
                if let Err(e) = self::compile::<X>(code, args, hash, true) {
                    tracing::debug!("failed to compile program: {e:?}");
                }
            });
        }

        // fallback to the interpreter
        Interpreter::invoke2(ctx, hash, code, args, gas, pc)
    }
}

impl Pvm for SpaceVM {}

/// Compile a program
pub fn compile<X: Argument>(
    code: Vec<u8>,
    args: Vec<u8>,
    hash: OpaqueHash,
    memcache: bool,
) -> Result<()> {
    match <pvmc::Module as ModuleLike>::new::<X>()?
        .compile(&parser::program::preimage(code, &args).expect("failed to preimage"))
    {
        Ok(module) => {
            if memcache {
                SPACEVM_MODULES.put(hash, Arc::new(module));
            }
        }
        Err(err) => {
            tracing::debug!("failed to compile program: {:?}", err);
        }
    }
    Ok(())
}
