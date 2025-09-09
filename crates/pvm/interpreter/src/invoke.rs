//! Invocation APIs of the interpreter

use crate::{
    pvmi::{self, ParsedProgram},
    Context, Interpreter,
};
use anyhow::Result;
use pvm::{
    host,
    score::{Gas, OpaqueHash},
    Argument, Invoked, Program, Reason,
};

impl Interpreter {
    /// Invoke a program with the given context
    pub fn invoke<X: Argument>(
        program: &Program,
        hash: OpaqueHash,
        ctx: X,
        gas: Gas,
        pc: usize,
    ) -> Result<Invoked<X>> {
        let blob = program.blob()?;
        let mut reader = blob.reader();
        let mut parsed = vec![None; reader.buffer.len()];
        while let Ok(instr) = reader.read() {
            parsed[instr.range.start] = Some(instr.clone());
        }

        let parsed = ParsedProgram {
            program: parsed,
            registers: program.registers,
            memory: program.memory.clone(),
            table: blob.jump_table.to_vec(),
        };

        pvmi::set(hash, parsed.clone());
        Self::invoke_parsed(parsed, ctx, gas, pc)
    }

    /// Invoke a program with the given context
    pub fn invoke_parsed<X: Argument>(
        program: ParsedProgram,
        mut ctx: X,
        gas: Gas,
        pc: usize,
    ) -> Result<Invoked<X>> {
        let initial_gas = gas;
        let mut interp = Interpreter {
            context: Context {
                gas: gas as i64,
                registers: program.registers,
                memory: program.memory.clone(),
            },
            pc,
            table: program.table,
            ..Default::default()
        };

        // interpret the program
        let program = program.program;
        loop {
            let Some(Some(instr)) = program.get(interp.pc) else {
                break;
            };

            interp.pc = instr.range.start;
            match interp.step(instr) {
                Reason::Continue => {
                    if let Some(target) = interp.jump.take() {
                        interp.pc = target;
                        continue;
                    }

                    interp.pc = instr.range.end;
                }
                Reason::HostCall(call) => {
                    let mut context = interp.context.ctx(&mut ctx);
                    let reason = host::call(call, &mut context);
                    interp.context.registers = context.registers;
                    if reason != Reason::Continue {
                        return Ok(interp.result(ctx, initial_gas, reason));
                    }
                    interp.pc = instr.range.end;
                }
                reason => {
                    return Ok(interp.result(ctx, initial_gas, reason));
                }
            }
        }

        interp.burn(1);

        Ok(interp.result(
            ctx,
            initial_gas,
            Reason::Panic("end of program".to_string()),
        ))
    }
}
