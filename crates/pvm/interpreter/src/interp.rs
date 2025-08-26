//! PolkaVM program interpreter

use crate::{Error, Interpreter};
use parser::{reader::Offset, Instruction, Visitor};
use pvm::Reason;

impl Interpreter {
    /// Step a single instruction.
    pub fn step(&mut self, instr: &Offset<Instruction>) -> Reason {
        if self.burn(1).is_err() {
            return Reason::OOG;
        }

        // charge extra gas for host calls based on the specification
        let extra_gas = match instr.value {
            Instruction::Ecalli(call_format) => {
                let call_number = call_format.imm0 as u32;
                match call_number {
                    11 => 10 + self.rget(9),
                    100 => 0,
                    _ => 10,
                }
            }
            _ => 0,
        };
        if extra_gas > 0 && self.burn(extra_gas).is_err() {
            return Reason::OOG;
        }

        // step the instruction
        let stepped = self.visit(instr.value, &instr.range);
        if let Err(e) = stepped {
            if self.burn(e.extra_gas()).is_err() {
                return Reason::OOG;
            }

            e.into()
        } else {
            Reason::Continue
        }
    }

    /// Read a value from memory
    pub fn read<V: pvm::Value>(&self, address: u32) -> crate::Result<V> {
        let bytes = self
            .memory
            .read_bytes(address, V::SIZE as u32)
            .map_err(|_e| Error::MemoryInaccessible {
                page: address / parser::PAGE_SIZE as u32,
            })?;
        V::from_bytes(&bytes).ok_or(Error::MemoryInaccessible {
            page: address / parser::PAGE_SIZE as u32,
        })
    }

    /// Read a value from memory at an offset
    pub fn read_offset<V: pvm::Value>(&self, address: u32, offset: u32) -> crate::Result<V> {
        let start = address.wrapping_add(offset);
        self.read(start)
    }

    /// Write a value to memory
    pub fn write<V: pvm::Value>(&mut self, address: u32, value: V) -> crate::Result<()> {
        self.memory
            .write_bytes(address, &value.to_vec())
            .map_err(|_e| Error::MemoryInaccessible {
                page: address / parser::PAGE_SIZE as u32,
            })
    }

    /// Write a value to memory at an offset
    pub fn write_offset<V: pvm::Value>(
        &mut self,
        address: u32,
        offset: u32,
        value: V,
    ) -> crate::Result<()> {
        let start = address.wrapping_add(offset);
        self.write(start, value)
    }

    /// Get the register value.
    pub fn rget(&self, reg: u8) -> u64 {
        self.registers[reg as usize]
    }

    /// Set the register value.
    pub fn rset(&mut self, reg: u8, value: u64) {
        self.registers[reg as usize] = value;
    }

    /// Allocate pages for sbrk
    pub fn allocate(&mut self, start_page: u32, count: u32) -> crate::Result<()> {
        self.memory
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

    /// Burn the gas.
    pub fn burn(&mut self, gas: u64) -> crate::Result<()> {
        if self.gas < gas as i64 {
            return Err(Error::OOG);
        }

        self.gas = self.gas.saturating_sub(gas as i64);
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
}
