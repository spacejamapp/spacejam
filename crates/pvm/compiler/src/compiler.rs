//! Clean block-based JIT compiler for PVM programs

use crate::{Module, JIT};
use anyhow::Result;
use pvm::{
    parser,
    score::{Gas, OpaqueHash},
    Argument, Invocation, Invoked, Program, State,
};

/// PVM compiler
pub struct Compiler;

impl Compiler {
    /// Compile entire program as a function
    pub fn compile(&mut self, program: &Program) -> Result<Module> {
        JIT::new()?.compile(program, None)
    }

    /// Compile entire program as a function with cache
    pub fn compile_with_cache<X: Argument>(
        &mut self,
        program: &Program,
        hash: Option<OpaqueHash>,
    ) -> Result<Module> {
        JIT::host::<X>()?.compile(program, hash)
    }
}

impl Invocation for Compiler {
    fn invoke2<X: Argument>(
        mut ctx: X,
        hash: OpaqueHash,
        code: Vec<u8>,
        args: Vec<u8>,
        gas: Gas,
        pc: usize,
    ) -> Invoked<X> {
        let program = parser::program::preimage(code, &args).expect("failed to preimage");
        let mut pvmc = JIT::host::<X>().expect("fix me later");
        let module = pvmc.compile(&program, Some(hash)).expect("fix me later");
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
