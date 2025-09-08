//! Invocation APIs of the interpreter

use crate::{Context, Interpreter};
use anyhow::Result;
use pvm::{host, score::Gas, Argument, Invoked, Program, Reason};

impl Interpreter {
    /// Invoke a program with the given context
    pub fn invoke<X: Argument>(
        program: &Program,
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
            ..Default::default()
        };
        let blob = program.blob()?;
        interp.table = blob.jump_table.to_vec();

        // interpret the program
        let mut reader = blob.reader().with_position(pc);
        loop {
            let Ok(instr) = reader.read() else {
                break;
            };

            interp.pc = instr.range.start;
            match interp.step(&instr) {
                Reason::Continue => {
                    if let Some(target) = interp.jump.take() {
                        interp.pc = target;
                        reader.set_position(target);
                    }

                    continue;
                }
                Reason::HostCall(call) => {
                    let mut context = interp.context.ctx(&mut ctx);
                    let reason = host::call(call, &mut context);
                    interp.context.registers = context.registers;
                    if reason != Reason::Continue {
                        return Ok(interp.result(ctx, initial_gas, reason));
                    }
                }
                reason => {
                    return Ok(interp.result(ctx, initial_gas, reason));
                }
            }
        }

        interp.pc = reader.position;
        interp.burn(1);
        Ok(interp.result(
            ctx,
            initial_gas,
            Reason::Panic("end of program".to_string()),
        ))
    }
}
