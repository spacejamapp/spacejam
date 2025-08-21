//! PolkaVM program interpreter
//!
//! TODOs:
//!
//! - [ ]: double check the update of program counter
//! - [ ]: double check the jump instruction (what's the exact PC)
//! - [ ]: introduce the sign / unsign transitionss

use crate::{Error, Memory, Register};
use pvm::Reason;

mod builder;
mod register;
mod visitor;

/// The interpreter for the polkavm program.
///
/// TODO: maybe use lifetime to save the cost for adpating the
/// invocation interfaces in the future.
#[derive(Default)]
pub struct Interpreter {
    /// The registers of the interpreter.
    /// ra = [0]
    /// sp = [1]
    ///  s = [5, 6]
    ///  a = [7, 8, 9, 10, 11]
    pub registers: [Register; 13],

    /// The gas limit of the interpreter.
    pub gas: u64,

    /// The reason of the exit-execution.
    pub reason: Reason,

    /// The memory of the interpreter.
    pub memory: Memory,

    /// The jump table of the program.
    pub table: Vec<u64>,

    /// The program counter.
    pub pc: usize,

    /// The jump target.
    pub jump: Option<usize>,
}

impl Interpreter {
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

    /// Allocate pages for sbrk
    pub fn allocate_memory_pages(&mut self, start_page: u32, count: u32) -> crate::Result<()> {
        self.memory
            .allocate(start_page, count)
            .map_err(|_e| Error::MemoryInaccessible { page: start_page })
    }

    /// Branch to the given target.
    fn branch(&mut self, offset: i32, jump: bool) -> crate::Result<()> {
        if jump {
            self.jump = Some((self.pc as i32 + offset) as usize);
        }

        Ok(())
    }

    /// Burn the gas.
    pub fn burn(&mut self, gas: u64) -> crate::Result<()> {
        if self.gas < gas {
            return Err(Error::OOG);
        }

        self.gas = self.gas.saturating_sub(gas);
        Ok(())
    }

    /// Dynamic jump to the given target.
    fn djump(&mut self, address: u32) -> crate::Result<()> {
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
