//! Cranelift JIT backend

use crate::{engine, host, Artifact};
use anyhow::Result;
use cranelift_codegen::Context;
use cranelift_jit::JITModule;
use cranelift_module::Module;
use pvm::{
    parser,
    score::{Gas, OpaqueHash},
    Argument, Invocation, Invoked, State,
};

/// Cranelift JIT module builder
pub struct Compiler {
    /// Cranelift JIT module builder
    pub module: JITModule,

    /// Cranelift codegen context
    pub ctx: Context,

    /// Artifact
    pub artifact: Artifact,
}

impl Compiler {
    /// Create new JIT module builder
    pub fn new() -> Result<Self> {
        let mut builder = engine::compilation()?;
        host::symbols::<pvm::Context<'_, (), crate::Memory>>(&mut builder);
        let module = JITModule::new(builder);
        Ok(Self {
            ctx: module.make_context(),
            module,
            artifact: Artifact::new()?,
        })
    }

    /// Create new JIT module builder for host functions
    pub fn host<X: Argument>() -> Result<Self> {
        let mut builder = engine::compilation()?;
        host::symbols::<X>(&mut builder);
        let module = JITModule::new(builder);
        Ok(Self {
            ctx: module.make_context(),
            module,
            artifact: Artifact::new()?,
        })
    }
}

impl Invocation for Compiler {
    fn invoke2<X: Argument>(
        mut ctx: X,
        _hash: OpaqueHash,
        code: Vec<u8>,
        args: Vec<u8>,
        gas: Gas,
        pc: usize,
    ) -> Invoked<X> {
        let program = parser::program::preimage(code, &args).expect("failed to preimage");
        let mut pvmc = Self::host::<X>().expect("fix me later");
        let module = pvmc.compile(&program).expect("fix me later");
        let mut context = pvm::Context {
            table: 0 as *const u8,
            registers: module.registers,
            gas: gas as i64,
            pc: pc as u64,
            memory: module.memory.clone(),
            ctx: &mut ctx,
        };

        let reason = module.execute(&mut context).expect("fix me later");
        Invoked {
            gas: gas - (context.gas.max(0) as u64),
            output: Default::default(),
            reason,
            state: State {
                pc: context.pc as usize,
                gas: context.gas,
                registers: context.registers,
                memory: context.memory.fill(&program.memory),
            },
            data: ctx,
        }
    }
}
