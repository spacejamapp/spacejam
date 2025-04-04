//! PVM interface implementation

use crate::{Interpreter, Memory};
use pvm::{Gas, Invocation, Reason, Stepped};
use pvm_parser::{Reader, Visitor};

impl Invocation for Interpreter {
    type Memory = Memory;

    /// Step the instruction.
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
        memory: Memory,
    ) -> Stepped<Memory, ()> {
        let mut state = pvm::State {
            memory: memory.clone(),
            registers,
            gas: gas as i64,
            pc,
        };

        let pc = pc as usize;
        let mut pvmi = Interpreter::default()
            .gas(gas)
            .registers(registers)
            .memory(memory)
            .pc(pc)
            .table(jump.to_vec());

        // check if the program counter is out of bounds
        state.gas -= 1;
        if pc >= instructions.len() {
            return Stepped::new(Reason::Panic("end of program".to_string()), state);
        }

        // read the instruction
        let mut reader = Reader::new(instructions, bitmask).with_position(pc);
        let instr = match reader.read() {
            Ok(instr) => instr,
            Err(e) => {
                tracing::error!("invalid instruction: {}", e);
                return Stepped::new(Reason::Panic(e.to_string()), state);
            }
        };

        // step the instruction
        tracing::trace!("0x{:06x} | {}", pc, instr.value);
        let stepped = pvmi.visit(instr.value);
        state.registers = pvmi.registers;
        state.memory = pvmi.memory.clone();

        // check if need to exit
        let reason = if let Err(e) = stepped {
            state.gas -= e.extra_gas() as i64;
            e.into()
        } else {
            if let Some(pos) = pvmi.jump.take() {
                state.pc = pos as u64;
            } else {
                state.pc = reader.position as u64;
            }
            Reason::Continue
        };

        Stepped::new(reason, state)
    }
}
