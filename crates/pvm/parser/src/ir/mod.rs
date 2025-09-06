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

        // unresolved jump targets and start position of a function
        let mut ujumps = BTreeMap::new();
        while !reader.eof() {
            let block = reader.read_block_ir()?;
            match block.control {
                Control::Call(target) => {
                    self.funcs.insert(target, FunctionRef::new(target));
                }
                Control::Jump(target) => {
                    ujumps.insert(block.range.start, target);
                }
                Control::Internal => {
                    ujumps.insert(block.range.start, block.range.end);
                }
            };

            self.blocks.insert(block.range.start, block);
        }

        self.parse_functions(&blob.jump_table)?;
        self.relocate(ujumps)
    }

    /// Verify if the IR is valid
    pub fn verify(&self, table: &[u64]) -> Result<()> {
        // check all jump table entries are in the IR
        let jsize = self
            .funcs
            .values()
            .map(|func| func.jump.len())
            .collect::<Vec<_>>()
            .iter()
            .sum::<usize>();
        if jsize != table.len() {
            anyhow::bail!("jump table length mismatch: {jsize} != {}", table.len());
        }

        // check all jumps are resolved
        //
        // 1. jump to local function
        // 2. jump to a function
        for (func_pc, func) in &self.funcs {
            for block_pc in &func.blocks {
                let Some(block) = self.blocks.get(block_pc) else {
                    eprintln!("block {func_pc}:{block_pc} not found");
                    continue;
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

                eprintln!("unresolved jump target {reachable} for block {func_pc}:{block_pc}, blocks count: {}", func.blocks.len());
            }
        }

        Ok(())
    }

    /// Handle the undiscovered jump targets
    fn relocate(&mut self, ujumps: BTreeMap<u64, u64>) -> Result<()> {
        let mut funcs = self
            .funcs
            .values()
            .map(|f| f.range.clone())
            .collect::<Vec<_>>();

        for (from, to) in ujumps {
            if self.funcs.contains_key(&to) {
                continue;
            }

            let mut to_split = None;
            for func in &funcs {
                if !func.contains(&from) {
                    continue;
                }

                if func.contains(&to) {
                    break;
                }

                to_split = Some(to);
                break;
            }

            let Some(to_split) = to_split else {
                continue;
            };

            // if we captured a splitting point, find the correct
            // function and split it
            for func in &funcs {
                if func.contains(&to_split) {
                    self.split(func.start, to_split)?;
                    break;
                }
            }

            // now we need to update funcs to apply this change
            // to other splitting points
            funcs = self
                .funcs
                .values()
                .map(|f| f.range.clone())
                .collect::<Vec<_>>();
        }

        Ok(())
    }

    /// Split out functions via the given entrypoint
    fn split(&mut self, func: u64, entry: u64) -> Result<()> {
        let func = self
            .funcs
            .get_mut(&func)
            .ok_or(anyhow::anyhow!("function {func} not found"))?;

        // update the range of the functions
        let mut next = FunctionRef::new(entry);
        next.range.end = func.range.end;
        func.range.end = entry;

        // scale the blocks and the jump table
        (func.jump, next.jump) = func.jump.iter().partition(|(_, pc)| **pc < entry);
        (func.blocks, next.blocks) = func.blocks.iter().partition(|&pc| *pc < entry);
        self.funcs.insert(entry, next);
        Ok(())
    }

    /// Parse the functions from blocks
    fn parse_functions(&mut self, table: &[u64]) -> Result<()> {
        let mut blocks = 0;
        let funcs = self.funcs.clone().into_iter().collect::<Vec<_>>();
        for (idx, (_, func)) in self.funcs.iter_mut().enumerate() {
            func.range.end = if let Some((_, next)) = funcs.get(idx + 1) {
                next.range.start
            } else if let Some((_, last)) = self.blocks.iter().last() {
                last.range.end
            } else {
                anyhow::bail!("no blocks found for function {}", func.range.start);
            };

            // update the blocks of the function
            for (_, block) in self.blocks.iter().skip(blocks) {
                if block.range.start >= func.range.end {
                    break;
                }

                if let Some(idx) = table.iter().position(|pc| *pc == block.range.start) {
                    func.jump.insert(idx as u32, block.range.start);
                }

                func.blocks.insert(block.range.start);
                blocks += 1;
            }
        }

        Ok(())
    }

    fn parse_exports(&mut self, reader: &mut Reader<'_>) -> Result<()> {
        while let Ok(instr) = reader.read() {
            let Offset {
                range,
                value: Instruction::Jump(fmt),
            } = instr
            else {
                reader.position = instr.range.start;
                break;
            };

            let instr = Instruction::Jump(fmt);
            let info = instr.info(range.clone());
            let target = (fmt.off0 as i64 + range.start as i64) as u64;
            self.funcs.insert(target, FunctionRef::new(target));
            self.exports.insert(range.start as u64, vec![]);
            self.blocks.insert(
                range.start as u64,
                Block {
                    range: range.start as u64..range.end as u64,
                    control: Control::Jump(target),
                    input: info.input.iter().cloned().collect(),
                    output: info.output.iter().cloned().collect(),
                    code: vec![(instr, info)],
                },
            );
        }

        Ok(())
    }
}
