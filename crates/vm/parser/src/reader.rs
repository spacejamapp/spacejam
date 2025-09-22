//! The binary reader.

use crate::{instruction::Instruction, opcode::Opcode, util};
use anyhow::Result;
use core::ops::Range;

/// The binary reader.
pub struct Reader<'r> {
    /// The instruction buffer to read from.
    pub buffer: &'r [u8],

    /// The bitmask of the instruction buffer.
    pub bitmask: &'r [u8],

    /// The current position in the buffer.
    pub position: usize,
}

impl<'r> Reader<'r> {
    /// Create a new binary reader.
    pub fn new(buffer: &'r [u8], bitmask: &'r [u8]) -> Self {
        Self {
            buffer,
            bitmask,
            position: 0,
        }
    }

    /// Check if the reader is at the end of the buffer.
    pub fn eof(&self) -> bool {
        self.position >= self.buffer.len()
    }

    /// Set the position of the reader.
    pub fn with_position(mut self, position: usize) -> Self {
        self.position = position;
        self
    }

    /// Set the position of the reader.
    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    /// Read an opcode.
    pub fn read_opcode(&mut self) -> Result<Opcode> {
        let opcode = Opcode::try_from(*self.buffer.get(self.position).ok_or(anyhow::anyhow!(
            "position {} not found in buffer",
            self.position
        ))?)?;
        self.position += 1;
        Ok(opcode)
    }

    /// Read an instruction.
    pub fn read(&mut self) -> Result<Offset<Instruction>> {
        let start = self.position;
        let opcode = self.read_opcode()?;

        // Get skip distance to next instruction
        let distance = util::skip(self.position, self.bitmask);
        let next = (self.position + distance).min(self.buffer.len());

        // Read instruction
        let buffer = &self.buffer[self.position..next];
        let instruction = opcode.instr(buffer);
        self.position = next;

        Ok(Offset {
            range: start..next,
            value: instruction,
        })
    }

    /// (A.4) Read a block-sequence of instructions.
    pub fn read_block(&mut self) -> Result<Vec<Offset<Instruction>>> {
        let mut block = Vec::new();
        while !self.eof() {
            let mut end_of_block = false;
            let instruction = self.read()?;
            if instruction.value.is_termination() {
                end_of_block = true;
            }

            block.push(instruction);
            if end_of_block {
                break;
            }
        }

        Ok(block)
    }
}

/// A wrapped value with an offset range.
#[derive(Debug, Clone)]
pub struct Offset<T> {
    /// The range.
    pub range: Range<usize>,

    /// The value.
    pub value: T,
}
