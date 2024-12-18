//! The instruction data.

use crate::{
    instruction::Instruction,
    reader::{Offset, Reader},
};
use anyhow::Result;
use core::ops::Range;

/// The instruction data.
pub struct InstructionData {
    /// The instructions.
    pub instructions: Vec<u8>,

    /// The bitmask of the instruction data.
    pub bitmask: Vec<u8>,

    /// The range of the instruction.
    pub range: Range<usize>,
}

impl InstructionData {
    /// Get the instruction reader.
    pub fn reader(&self) -> InstructionReader {
        InstructionReader {
            bitmask: &self.bitmask,
            reader: Reader::new(&self.instructions, self.range.start),
        }
    }
}

/// The instruction reader.
pub struct InstructionReader<'r> {
    /// The buffer.
    bitmask: &'r [u8],

    /// The reader.
    reader: Reader<'r>,
}

impl<'r> InstructionReader<'r> {
    /// Read an instruction.
    pub fn read(&mut self) -> Result<Offset<Instruction>> {
        self.reader.read_instr(self.bitmask)
    }
}

impl<'r> core::ops::Deref for InstructionReader<'r> {
    type Target = Reader<'r>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}
