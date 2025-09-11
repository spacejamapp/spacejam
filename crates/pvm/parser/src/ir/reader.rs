//! IR blocks reader

use std::collections::BTreeSet;

use crate::instruction::InstructionType;
use crate::ir::{Block, Control};
use crate::Reader;
use anyhow::Result;

impl Reader<'_> {
    /// Read a IR format block
    pub fn read_block_ir(&mut self) -> Result<Block> {
        let mut block = Block::default();
        block.range.start = self.position as u64;
        let mut input = BTreeSet::new();
        let mut output = BTreeSet::new();
        while !self.eof() {
            let instr = self.read()?;
            let range = instr.range;
            let info = instr.value.info(range);
            block.code.push((instr.value, info.clone()));
            block.range.end = info.range.end as u64;

            // update the register info of the block
            {
                for reg in info.input {
                    if !output.contains(&reg) {
                        input.insert(reg);
                    }
                }

                for reg in info.output {
                    output.insert(reg);
                }
            }

            // Check if reach the termination instruction
            match info.ty {
                InstructionType::StaticJump(pc) => {
                    block.control = Control::Jump(pc);
                    break;
                }
                InstructionType::Call(pc) => {
                    block.control = Control::Call(pc);
                    break;
                }
                InstructionType::DynamicJump => {
                    break;
                }
                _ => continue,
            }
        }

        block.output = output;
        block.input = input;
        Ok(block)
    }
}
