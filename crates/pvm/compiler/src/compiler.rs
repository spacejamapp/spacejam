//! Cranelift JIT backend

use crate::{Artifact, Memory, engine, host};
use anyhow::Result;
use cranelift_jit::JITModule;
use pvm::{
    Argument, Invocation, Invoked, State, parser,
    score::{Gas, OpaqueHash},
};

/// Cranelift JIT module builder
pub struct Compiler {
    /// Cranelift JIT module builder
    pub module: JITModule,

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
        let pvmc = Self::host::<X>().expect("fix me later");
        let module = pvmc.compile(&program).expect("fix me later");
        let mut context = pvm::Context {
            registers: program.registers,
            gas: gas as i64,
            memory: Memory::new(&program.memory).expect("failed to create memory"),
            ctx: &mut ctx,
        };

        let reason = module
            .execute(&mut context, pc as u64)
            .expect("fix me later");
        Invoked {
            gas: gas - (context.gas.max(0) as u64),
            output: Default::default(),
            reason,
            state: State {
                pc: 0,
                gas: context.gas,
                registers: context.registers,
                memory: context.memory.fill(&program.memory),
            },
            data: ctx,
        }
    }
}
