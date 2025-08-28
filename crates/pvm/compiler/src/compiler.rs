//! Clean block-based JIT compiler for PVM programs

use crate::{Module, JIT};
use anyhow::Result;
use pvm::{score::Gas, Argument, Invocation, Invoked, Program, State};

/// PVM compiler
pub struct Compiler;

impl Compiler {
    /// Create new JIT compiler
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Compile entire program as a function
    pub fn compile(&mut self, program: &Program) -> Result<Module> {
        JIT::new()?.compile(program)
    }
}

impl Invocation for Compiler {
    fn invoke2<X: Argument>(program: &Program, mut ctx: X, gas: Gas, pc: usize) -> Invoked<X> {
        let mut pvmc = JIT::host::<X>().expect("fix me later");
        let module = pvmc.compile(program).expect("fix me later");
        let mut context = pvm::Context {
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
