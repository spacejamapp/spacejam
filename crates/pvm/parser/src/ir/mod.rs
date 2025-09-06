//! Jastime IR

use crate::{reader::Offset, Instruction, ProgramBlob, Reader};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
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

        // undiscovered jump targets and start position of a function
        let mut ujumps = BTreeSet::new();
        let mut start = reader.position as u64;
        while !reader.eof() {
            let pc = reader.position as u64;
            let block = reader.read_block_ir()?;

            // check the control flow type of the block
            let jump = match block.control {
                Control::Call(target) => {
                    self.funcs.insert(target, FunctionRef::new(target));
                    target
                }
                Control::Jump(target) => target,
                Control::Internal => block.range.end,
            };

            if !self.funcs.contains_key(&jump) {
                ujumps.insert(jump);
            }

            // check if this block is a dynamic jump target
            let func = self.func(start)?;
            if let Some(index) = blob.jump_table.iter().position(|&x| x == block.range.start) {
                func.jump.insert(index as u32, pc);
            }

            // insert the block to function
            func.blocks.insert(pc);
            self.blocks.insert(pc, block);

            // check if we reach a new function boundary
            if self.funcs.contains_key(&pc) {
                self.func(start)?.range.end = pc;
                start = pc;
            }
        }

        let _ = self.relocate(ujumps);
        Ok(())
    }

    /// Verify if the IR is valid
    pub fn verify(&self) -> Result<()> {
        for (func_pc, func) in &self.funcs {
            for block_pc in &func.blocks {
                let Some(block) = self.blocks.get(block_pc) else {
                    anyhow::bail!("block {func_pc}:{block_pc} not found");
                };

                let reachable = block.reachable();
                if self.funcs.contains_key(&reachable) {
                    continue;
                }

                if func.range.contains(&reachable) {
                    continue;
                }

                if func.jump.values().any(|pc| *pc == reachable) {
                    continue;
                }

                anyhow::bail!("unresolved jump target {reachable} for block {func_pc}:{block_pc}");
            }
        }

        Ok(())
    }

    /// Handle the undiscovered jump targets
    fn relocate(&mut self, ujumps: BTreeSet<u64>) -> Result<()> {
        for jump in ujumps {
            let funcs = self
                .funcs
                .values()
                .map(|f| f.range.clone())
                .collect::<Vec<_>>();

            for func in funcs {
                if !func.contains(&jump) {
                    continue;
                }

                self.split(func.start, jump)?;
            }
        }
        Ok(())
    }

    /// Split out functions via the given entrypoint
    fn split(&mut self, func: u64, entry: u64) -> Result<()> {
        let func = self
            .funcs
            .get_mut(&func)
            .ok_or(anyhow::anyhow!("function {func} not found"))?;
        let mut next = FunctionRef::new(entry);

        // update the range of the functions
        next.range.end = func.range.end;
        func.range.end = entry;

        // scale the blocks and the jump table
        (func.jump, next.jump) = func.jump.iter().partition(|(_, pc)| **pc <= entry);
        (func.blocks, next.blocks) = func.blocks.iter().partition(|&pc| *pc <= entry);
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
            let target = (fmt.off0 as i64 + range.start as i64) as u64;
            self.funcs.insert(target, FunctionRef::new(target));
        }

        self.funcs
            .insert(reader.position as u64, FunctionRef::default());
        Ok(())
    }
}
