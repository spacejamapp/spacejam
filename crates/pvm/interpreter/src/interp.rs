//! PolkaVM program interpreter

use crate::{Error, Interpreter};
use parser::{format, reader::Offset, Instruction, Visitor};
use pvm::{Argument, Invoked, Reason};

impl Interpreter {
    /// Allocate pages for sbrk
    pub fn allocate(&mut self, start_page: u32, count: u32) -> crate::Result<()> {
        self.context
            .memory
            .allocate(start_page, count)
            .map_err(|_e| Error::MemoryInaccessible { page: start_page })
    }

    /// Branch to the given target.
    pub fn branch(&mut self, offset: i32, jump: bool) -> crate::Result<()> {
        if jump {
            self.jump = Some((self.pc as i32 + offset) as usize);
        }

        Ok(())
    }

    /// Dynamic jump to the given target.
    pub fn djump(&mut self, address: u32) -> crate::Result<()> {
        if address == u32::MAX - u16::MAX as u32 {
            return Err(Error::Terminate);
        }

        if address == 0
            || address > self.table.len() as u32 * pvm::JUMP_ALIGNMENT_FACTOR
            || address % 2 != 0
        {
            tracing::error!(
                "invalid dynamic jump, address: {}, table len: {}",
                address,
                self.table.len()
            );
            return Err(Error::InvalidDynamicJump);
        }

        let index = address as usize / 2 - 1;
        let Some(target) = self.table.get(index) else {
            tracing::error!(
                "invalid dynamic jump, index: {}, table len: {}",
                index,
                self.table.len()
            );
            return Err(Error::InvalidDynamicJump);
        };

        self.jump = Some(*target as usize);
        Ok(())
    }

    /// Read a value from memory
    pub fn read<V: pvm::Value>(&self, address: u32) -> crate::Result<V> {
        let bytes = self
            .context
            .memory
            .read_bytes(address, V::SIZE as u32)
            .map_err(|_e| Error::MemoryInaccessible {
                page: address / parser::PAGE_SIZE as u32,
            })?;
        V::from_bytes(&bytes).ok_or(Error::MemoryInaccessible {
            page: address / parser::PAGE_SIZE as u32,
        })
    }

    /// Get the result of the interpreter
    pub fn result<X: Argument>(&self, data: X, gas: u64, reason: Reason) -> Invoked<X> {
        Invoked {
            gas: gas - self.context.gas.max(0) as u64,
            output: self.output(),
            reason,
            data,
            state: self.state(),
        }
    }

    /// Step a single instruction.
    pub fn step(&mut self, instr: &Offset<Instruction>) -> Reason {
        let gas = match instr.value {
            Instruction::Ecalli(format::I { imm0: call }) => {
                let gas = match call {
                    20 => 11 + self.rget(9),
                    100 => 1,
                    _ => 11,
                };
                gas
            },
            _ => 1,
        };

        self.burn(gas);
        if self.context.gas < 0 {
            return Reason::OOG;
        }

        let stepped = self.visit(instr.value, &instr.range);
        if let Err(e) = stepped {
            e.into()
        } else {
            Reason::Continue
        }
    }

    /// Write a value to memory
    pub fn write<V: pvm::Value>(&mut self, address: u32, value: V) -> crate::Result<()> {
        self.context
            .memory
            .write_bytes(address, &value.to_vec())
            .map_err(|_e| Error::MemoryInaccessible {
                page: address / parser::PAGE_SIZE as u32,
            })
    }
}
