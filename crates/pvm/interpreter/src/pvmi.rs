//! PVM interface implementation

use crate::{Interpreter, Memory};
use parser::{reader::Offset, Instruction, Reader, Visitor};
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
        if pvmi.pc == instructions.len() {
            let reason = if pvmi.burn(1).is_err() {
                Reason::OOG
            } else {
                Reason::Panic("end of program".to_string())
            };

            return Stepped::new(reason, pvmi.into());
        }

        // read the instruction
        let mut reader = Reader::new(instructions, bitmask).with_position(pc);
        let block = match reader.read_block() {
            Ok(block) => block,
            Err(e) => {
                tracing::error!("invalid instruction: {}", e);
                if pvmi.burn(1).is_err() {
                    return Stepped::new(Reason::OOG, pvmi.into());
                }

                return Stepped::new(Reason::Panic(e.to_string()), pvmi.into());
            }
        };

        // process the block sequence of instructions
        tracing::trace!("Compiling block:");
        tracing::trace!(
            "charge_gas: {} ({} -> {})",
            block.len(),
            pvmi.gas,
            pvmi.gas - block.len() as u64,
        );
        let mut reason = Reason::Continue;
        for instr in block {
            tracing::trace!("{:6} | {}", instr.range.start, instr.value);
            let next = instr.range.end;
            reason = pvmi.step_single(instr);
            if !matches!(reason, Reason::Continue | Reason::HostCall(_)) {
                break;
            }

            if let Some(pos) = pvmi.jump.take() {
                pvmi.pc = pos;
            } else {
                pvmi.pc = next;
            }

            if matches!(reason, Reason::HostCall(_)) {
                break;
            }
        }

        Stepped::new(reason, pvmi.into())
    }
}

impl Interpreter {
    /// Step a single instruction.
    fn step_single(&mut self, instr: Offset<Instruction>) -> Reason {
        // check if the gas has been exhausted
        if self.burn(1).is_err() {
            return Reason::OOG;
        }

        // charge extra gas for host calls based on the specification
        let extra_gas = match instr.value {
            Instruction::Ecalli(call_format) => {
                let call_number = call_format.imm0 as u32;
                match call_number {
                    // transfer: Gas cost is 10 + ω₉ (10 + register 9 value)
                    11 => 10 + self.registers[9],
                    // log: Gas cost is 0 as defined in JIP-1
                    100 => 0,
                    // All other host calls: Gas cost is 10
                    _ => 10,
                }
            }
            _ => 0,
        };
        if extra_gas > 0 && self.burn(extra_gas).is_err() {
            return Reason::OOG;
        }

        // step the instruction
        let stepped = self.visit(instr.value);
        if let Err(e) = stepped {
            if self.burn(e.extra_gas()).is_err() {
                return Reason::OOG;
            }

            e.into()
        } else {
            Reason::Continue
        }
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
