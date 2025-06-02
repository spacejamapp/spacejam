//! PVM interface implementation

use crate::{Interpreter, Memory};
use parser::{Reader, Visitor};
use pvm::{Gas, Invocation, Reason, Stepped};

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
        let pc = pc as usize;
        let mut pvmi = Interpreter::default()
            .gas(gas)
            .registers(registers)
            .memory(memory)
            .pc(pc)
            .table(jump.to_vec());

        // check if the program counter is out of bounds
        pvmi.gas -= 1;
        if pc >= instructions.len() {
            return Stepped::new(Reason::Panic("end of program".to_string()), pvmi.into());
        }

        // read the instruction
        let mut reader = Reader::new(instructions, bitmask).with_position(pc);
        let instr = match reader.read() {
            Ok(instr) => instr,
            Err(e) => {
                tracing::error!("invalid instruction: {}", e);
                return Stepped::new(Reason::Panic(e.to_string()), pvmi.into());
            }
        };

        // step the instruction
        tracing::trace!("{:6} | {} | {:?}", pc, instr.value, registers);
        let stepped = pvmi.visit(instr.value);
        let reason = if let Err(e) = stepped {
            pvmi.gas = pvmi.gas.saturating_sub(e.extra_gas());

            // For host calls, advance the PC to the next instruction before triggering the call
            if matches!(e, crate::Error::HostCall(_)) {
                if let Some(pos) = pvmi.jump.take() {
                    pvmi.pc = pos;
                } else {
                    pvmi.pc = reader.position;
                }
            }

            e.into()
        } else {
            if let Some(pos) = pvmi.jump.take() {
                pvmi.pc = pos;
            } else {
                pvmi.pc = reader.position;
            }
            Reason::Continue
        };

        Stepped::new(reason, pvmi.into())
    }
}

impl From<Interpreter> for pvm::State<Memory> {
    fn from(interp: Interpreter) -> Self {
        pvm::State {
            memory: interp.memory,
            registers: interp.registers,
            gas: interp.gas as i64,
            pc: interp.pc as u64,
        }
    }
}
