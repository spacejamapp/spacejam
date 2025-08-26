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
        let mut interp = {
            let mut interp = Interpreter::default();
            interp.gas = gas as i64;
            interp.pc = pc;
            interp.registers = [0; 13];
            interp.memory = program.memory.clone();
            interp
        };

        // deblob the program
        let blob = program.blob()?;
        interp.table = blob.jump_table.to_vec();
        let mut reader = blob.reader();
        loop {
            let block = reader.read_block()?;
            if block.is_empty() {
                break;
            }

            for instr in block {
                match interp.step(&instr) {
                    Reason::Continue => continue,
                    Reason::HostCall(call) => {
                        let Stepped {
                            reason,
                            state,
                            data,
                        } = host::call(call, interp.state(), ctx);
                        interp.set_state(state);
                        ctx = data;
                        if reason != Reason::Continue {
                            return Ok(Received {
                                gas: gas - (interp.gas.max(0) as u64),
                                output: interp.output(),
                                reason,
                                data: ctx,
                            });
                        }
                    }
                    reason => {
                        return Ok(Received {
                            gas: gas - (interp.gas.max(0) as u64),
                            output: interp.output(),
                            reason,
                            data: ctx,
                        })
                    }
                }
            }

            break;
        }

        Ok(Received {
            gas: gas - (interp.gas.max(0) as u64),
            output: interp.output(),
            reason: Reason::Halt,
            data: ctx,
        })
    }
}
