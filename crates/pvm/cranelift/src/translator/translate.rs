//! translation utils

use crate::{translator::Block, Translator};
use anyhow::Result;
use parser::Visitor;

impl Translator<'_> {
    /// Translate block and check termination
    pub fn translate_block(&mut self, block: &Block) -> Result<()> {
        // Translate all instructions in this block
        for instruction in &block.instructions {
            let pc = instruction.range.start;
            tracing::trace!("translating PC {} instruction {:?}", pc, instruction.value);

            if let Err(e) = self.visit(instruction.value, &instruction.range) {
                tracing::warn!("Instruction translation failed at PC {}: {}", pc, e);
            }
        }

        // Handle block termination with native Cranelift control flow
        //
        // This is only for the tests that has incomplete blocks.
        if let Some(last) = block.instructions.last() {
            if !last.value.is_termination() {
                self.return_continue_with_pc(block.end as u64)?;
            }
        }
        Ok(())
    }

    /// Calculate the length of a PVM instruction at the given PC
    /// This is crucial for correct PC advancement when branches are not taken
    /// Note: This method still needs parsing for individual instruction length calculations
    /// when called during branch instruction processing. This is unavoidable as instruction
    /// lengths are needed during compilation, not just during initial analysis.
    pub fn get_instruction_length(&self, pc: usize) -> Result<usize> {
        if self.program.is_empty() {
            return Err(anyhow::anyhow!(
                "Program data not available for instruction length calculation"
            ));
        }

        let blob = parser::program::deblob(&self.program)?;
        let mut reader = blob.reader();
        reader.set_position(pc);

        if reader.eof() {
            return Err(anyhow::anyhow!("PC {} beyond program bounds", pc));
        }

        let start_pos = reader.position;
        // Use read_block and take the first instruction to calculate length
        let block_instructions = reader.read_block()?;
        if block_instructions.is_empty() {
            return Err(anyhow::anyhow!("No instruction found at PC {}", pc));
        }
        let instruction = &block_instructions[0];
        let end_pos = instruction.range.end;

        Ok(end_pos - start_pos)
    }
}
