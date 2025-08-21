//! PVM interface implementation

use crate::Interpreter;
use parser::Reader;
use pvm::{Gas, Invocation, Reason, State, Stepped};

impl Invocation for Interpreter {
    /// Step the instruction.
    // #[tracing::instrument(skip_all, target = "pvmi")]
    fn step(
        // (c) The instruction data
        instructions: &[u8],
        // (k) The bitmap of the instruction data
        bitmask: &[u8],
        // (j) The jump table
        jump: &[u64],
        // (ı) The current program counter
        pc: u64,
        // (ϱ) The gas
        gas: Gas,
        // (ω) The registers
        registers: [u64; 13],
        // (µ) The memory
        memory: parser::Memory,
    ) -> Stepped<()> {
        let pc = pc as usize;
        let mut pvmi = Interpreter::new(
            State {
                pc,
                gas: gas as i64,
                registers,
                memory,
            },
            jump.to_vec(),
        );

        // check if the program counter is out of bounds
        if pvmi.state.pc == instructions.len() {
            let reason = if pvmi.burn(1).is_err() {
                Reason::OOG
            } else {
                Reason::Panic("end of program".to_string())
            };

            return Stepped::new(reason, pvmi.state);
        }

        // read the instruction
        let mut reader = Reader::new(instructions, bitmask).with_position(pc);
        let block = match reader.read_block() {
            Ok(block) => block,
            Err(e) => {
                tracing::error!("invalid instruction: {}", e);
                if pvmi.burn(1).is_err() {
                    return Stepped::new(Reason::OOG, pvmi.state);
                }

                return Stepped::new(Reason::Panic(e.to_string()), pvmi.state);
            }
        };

        // process the block sequence of instructions
        let mut reason = Reason::Continue;
        for instr in block {
            let next = instr.range.end;
            reason = pvmi.step(&instr);
            tracing::trace!(
                "pos={:<6} {:<20} gas={:<6} regs={:?}",
                instr.range.start,
                instr.value.to_string(),
                pvmi.state.gas,
                pvmi.state.registers
            );

            if !matches!(reason, Reason::Continue | Reason::HostCall(_)) {
                break;
            }

            if let Some(pos) = pvmi.jump.take() {
                pvmi.state.pc = pos;
            } else {
                pvmi.state.pc = next;
            }

            if matches!(reason, Reason::HostCall(_)) {
                break;
            }
        }

        Stepped::new(reason, pvmi.state)
    }
}
