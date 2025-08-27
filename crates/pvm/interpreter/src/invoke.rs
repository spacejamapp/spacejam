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

        // deblob the program
        let blob = program.blob()?;
        interp.table = blob.jump_table.to_vec();
        let mut reader = blob.reader().with_position(pc);
        loop {
            let block = reader.read_block()?;
            if block.is_empty() {
                break;
            }

            for instr in block {
                tracing::trace!(
                    "pos={:<6} {:<20} gas={:<6} regs={:?}",
                    instr.range.start,
                    instr.value.to_string(),
                    interp.context.gas,
                    interp.context.registers
                );
                interp.pc = instr.range.start;
                match interp.step(&instr) {
                    Reason::Continue => {
                        if let Some(target) = interp.jump.take() {
                            interp.pc = target;
                            reader.set_position(target);
                            break;
                        }

                        continue;
                    }
                    Reason::HostCall(call) => {
                        let mut context = interp.context.ctx(ctx);
                        let reason = host::call(call, &mut context);
                        ctx = context.ctx;
                        if reason != Reason::Continue {
                            return Ok(interp.result(ctx, initial_gas, reason));
                        }
                    }
                    reason => {
                        return Ok(interp.result(ctx, initial_gas, reason));
                    }
                }
            }
        }

        interp.pc = reader.position;
        let _ = interp.burn(1);
        Ok(interp.result(
            ctx,
            initial_gas,
            Reason::Panic("end of program".to_string()),
        ))
    }
}
