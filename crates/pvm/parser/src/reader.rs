//! The binary reader.

use crate::{instruction::Instruction, opcode::Opcode};
use anyhow::Result;
use core::ops::Range;

/// The binary reader.
pub struct Reader<'r> {
    /// The buffer to read from.
    pub buffer: &'r [u8],

    /// The current position in the buffer.
    pub position: usize,

    /// The original offset of the buffer.
    pub original_offset: usize,
}

impl<'r> Reader<'r> {
    /// Create a new binary reader.
    pub fn new(buffer: &'r [u8], original_offset: usize) -> Self {
        Self {
            buffer,
            position: 0,
            original_offset,
        }
    }

    /// Read an instruction.
    pub fn read_instr(&mut self, bitmask: &[u8]) -> Result<Offset<Instruction>> {
        let next_instr = self.next_instr(bitmask);
        let opcode = Opcode::try_from(self.buffer[self.position])?;
        let instruction = opcode.instr(&self.buffer[self.position + 1..next_instr])?;
        self.position = next_instr;

        Ok(Offset {
            range: self.position..next_instr,
            value: instruction,
        })
    }

    /// Calculate the position of the next instruction.
    ///
    /// using the `skip` function defined in graypaper.
    fn next_instr(&self, bitmask: &[u8]) -> usize {
        for j in 0..24 {
            // Check if next position is an opcode
            if self.position + 1 + j >= bitmask.len() || bitmask[self.position + 1 + j] == 1 {
                return j + self.position;
            }
        }

        24 + self.position
    }
}

/// A wrapped value with an offset range.
pub struct Offset<T> {
    /// The range.
    pub range: Range<usize>,

    /// The value.
    pub value: T,
}
