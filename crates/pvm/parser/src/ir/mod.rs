//! Jastime IR

use crate::{reader::Offset, Instruction, ProgramBlob, Reader};
use anyhow::Result;
use std::collections::BTreeMap;

pub use {
    block::{Block, Control},
    func::{Export, Function, FunctionRef},
};

mod block;
mod func;
mod reader;

/// Jastime IR
#[derive(Debug, Clone, Default)]
pub struct IR {
    /// The exports of the program
    pub exports: BTreeMap<u64, Vec<u64>>,

    /// The functions of the program
    pub funcs: BTreeMap<u64, FunctionRef>,

    /// The basic blocks of the program
    pub blocks: BTreeMap<u64, Block>,
}

impl IR {
    /// Get an export from entry program counter
    pub fn export(&self, _entry: u64) -> Option<Export> {
        None
    }

    /// Get a function from program counter
    ///
    /// or mb we just need a interfaces like functions?
    pub fn function(&self, _entry: u64) -> Option<Function> {
        None
    }

    /// Parse the IR from a program blob
    pub fn parse(&mut self, blob: &ProgramBlob<'_>) -> Result<()> {
        let mut reader = blob.reader();
        self.parse_exports(&mut reader)?;

        // start position of a function
        let mut start = reader.position as u64;
        while !reader.eof() {
            let pc = reader.position as u64;
            let block = reader.read_block_ir()?;
            if let Control::Call(pc) = block.control {
                self.funcs.insert(pc, FunctionRef::new(pc));
            }

            // check if this block is a dynamic jump target
            let func = self.func(start)?;
            if let Some(index) = blob
                .jump_table
                .iter()
                .position(|&x| x == block.range.start as u64)
            {
                func.jump.insert(index as u32, pc);
            }

            // insert the block to function
            func.blocks.insert(pc);
            self.blocks.insert(pc, block);

            // check if we reach a new function boundary
            if self.funcs.get(&pc).is_some() {
                self.func(start)?.range.end = pc;
                start = pc;
            }
        }

        Ok(())
    }

    /// Get a function from program counter
    fn func(&mut self, pc: u64) -> Result<&mut FunctionRef> {
        self.funcs
            .get_mut(&pc)
            .ok_or(anyhow::anyhow!("function not found"))
    }

    fn parse_exports(&mut self, reader: &mut Reader<'_>) -> Result<()> {
        while let Ok(Offset {
            range,
            value: Instruction::Jump(fmt),
        }) = reader.read()
        {
            self.exports.insert(range.start as u64, vec![]);
            let target = (fmt.imm0 as i64 + range.start as i64) as u64;
            self.funcs.insert(target, FunctionRef::new(target));
        }

        self.funcs
            .insert(reader.position as u64, FunctionRef::default());
        Ok(())
    }
}
