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
mod resolver;
mod verifier;

/// Jastime IR
#[derive(Debug, Clone, Default)]
pub struct IR {
    /// The exports of the program
    pub exports: BTreeMap<u64, u64>,

    /// The functions of the program
    pub funcs: BTreeMap<u64, FunctionRef>,

    /// The basic blocks of the program
    pub blocks: BTreeMap<u64, Block>,
}

impl IR {
    /// Get an export from entry program counter
    ///
    /// Experimental feature
    pub fn export(&self, entry: u64) -> Option<Export> {
        let mut export = Export {
            entry,
            main: self.func_at(entry)?,
            funcs: BTreeMap::new(),
        };

        // collect the dynamic library functions
        let known = self
            .funcs
            .iter()
            .filter_map(|(_, fref)| {
                if fref.jump.is_empty() {
                    None
                } else {
                    Some(fref.blocks.clone())
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        // collect the dynamic library functions
        export.funcs = self.search(&known, &[], true).ok()?;
        export.funcs.extend(
            self.search(
                &export.main.blocks.keys().cloned().collect::<Vec<_>>(),
                &known,
                false,
            )
            .ok()?,
        );

        Some(export)
    }

    /// Get a function at a block
    pub fn func_at(&self, entry: u64) -> Option<Function> {
        let block = self.blocks.get(&entry)?;
        let func = Function {
            range: block.range.clone(),
            jump: BTreeMap::new(),
            blocks: BTreeMap::from([(entry, block.clone())]),
        };

        Some(func)
    }

    /// Get a function from program counter
    pub fn function(&self, entry: u64) -> Option<(Function, bool)> {
        let funcref = self.funcs.get(&entry)?;
        let is_dyn = !funcref.jump.is_empty();
        let mut func = funcref.func();
        for block in &funcref.blocks {
            func.blocks.insert(*block, self.blocks.get(block)?.clone());
        }

        Some((func, is_dyn))
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
                    ujumps
                        .entry(target)
                        .or_insert(vec![])
                        .push(block.range.start);
                }
                Control::Internal => {
                    /*  ujumps
                    .entry(block.range.end)
                    .or_insert(vec![])
                    .push(block.range.start); */
                }
            };

            self.blocks.insert(block.range.start, block);
        }

        self.parse_functions(&blob.jump_table)?;
        self.resolve(ujumps)
    }

    /// Search the functions from the blocks
    pub fn search(
        &self,
        blocks: &[u64],
        known: &[u64],
        is_dyn: bool,
    ) -> Result<BTreeMap<u64, Function>> {
        let mut funcs = BTreeMap::new();
        // 1. Collect reachable functions from main function
        let mut visited = BTreeSet::from_iter(known.iter().cloned());
        let mut queue = Vec::new();
        for block in blocks {
            let block = self
                .blocks
                .get(block)
                .ok_or(anyhow::anyhow!("block {block} not found"))?;
            let reachable_pc = block.reachable();
            if self.funcs.contains_key(&reachable_pc) && visited.insert(reachable_pc) {
                queue.push(reachable_pc);
            }
        }

        // 2. Process queue until no more functions are found
        while let Some(func_pc) = queue.pop() {
            if let Some((func, is_dyn_local)) = self.function(func_pc) {
                if !is_dyn && is_dyn_local {
                    continue;
                }

                for block in func.blocks.values() {
                    let reachable = block.reachable();
                    if self.funcs.contains_key(&reachable) && visited.insert(reachable) {
                        queue.push(reachable);
                    }
                }
                funcs.insert(func_pc, func);
            }
        }
        Ok(funcs)
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
            self.exports.insert(range.start as u64, target);
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
