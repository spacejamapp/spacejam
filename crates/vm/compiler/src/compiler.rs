//! Cranelift JIT backend

use crate::{Memory, ModuleLike};
use pvm::{
    Argument, Invocation, Invoked, Pvm, State, parser,
    score::{Gas, OpaqueHash},
};

/// Cranelift JIT module builder
pub struct Compiler;

impl Pvm for Compiler {}

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
        let pvmc = <crate::Module as ModuleLike>::new::<X>().expect("fix me later");
        let module = pvmc.compile(&program).expect("fix me later");
        let mut context = pvm::Context {
            registers: program.registers,
            gas: gas as i64,
            memory: Memory::new(hash, &program.memory).expect("failed to create memory"),
            ctx: &mut ctx,
        };

        let reason = module
            .execute(&mut context, pc as u64)
            .expect("fix me later");
        let output = crate::trap::with(|| context.acc_output()).unwrap_or_default();
        Invoked {
            gas: gas - (context.gas.max(0) as u64),
            output,
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
