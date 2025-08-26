//! Invocation APIs of the interpreter

use crate::Interpreter;
use anyhow::Result;
use pvm::{host, Argument, Gas, Program, Reason, Received, Stepped};

impl Interpreter {
    /// Invoke a program with the given context
    pub fn invoke<X: Argument>(
        program: &Program,
        mut ctx: X,
        gas: Gas,
        pc: usize,
    ) -> Result<Received<X>> {
        let initial_gas = gas;
        let mut interp = {
            let mut interp = Interpreter::default();
            interp.gas = gas as i64;
            interp.pc = pc;
            interp.registers = program.registers;
            interp.memory = program.memory.clone();
            interp
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
                    interp.gas,
                    interp.registers
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
                        let Stepped {
                            reason,
                            state,
                            data,
                        } = host::call(call, interp.state(), ctx);
                        interp.set_state(state);
                        ctx = data;
                        if reason != Reason::Continue {
                            let consumed_gas = initial_gas - interp.gas.max(0) as u64;
                            return Ok(Received {
                                gas: consumed_gas,
                                output: interp.output(),
                                reason,
                                data: ctx,
                            });
                        }
                    }
                    reason => {
                        let consumed_gas = initial_gas - interp.gas.max(0) as u64;
                        return Ok(Received {
                            gas: consumed_gas,
                            output: interp.output(),
                            reason,
                            data: ctx,
                        });
                    }
                }
            }
        }

        let consumed_gas = initial_gas - interp.gas.max(0) as u64;
        Ok(Received {
            gas: consumed_gas,
            output: interp.output(),
            reason: Reason::Halt,
            data: ctx,
        })
    }
}
