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

    /// Check if the reader is at the end of the buffer.
    pub fn eof(&self) -> bool {
        self.position >= self.buffer.len()
    }

    /// Read an opcode.
    pub fn read_opcode(&mut self) -> Result<Opcode> {
        let opcode = Opcode::try_from(self.buffer[self.position])?;
        self.position += 1;
        Ok(opcode)
    }

    /// Read an instruction.
    pub fn read_instr(&mut self, bitmask: &[u8]) -> Result<Offset<Instruction>> {
        let start = self.position;
        let opcode = self.read_opcode()?;

        // Get skip distance to next instruction
        let next_instr = self.next_instr(bitmask);

        // Read instruction
        let buffer = &self.buffer[self.position..next_instr];
        let instruction = opcode.instr(buffer);
        self.position = next_instr;

        Ok(Offset {
            range: start..next_instr,
            value: instruction,
        })
    }

    /// Find the next instruction.
    fn next_instr(&self, bitmask: &[u8]) -> usize {
        let mut pc = self.position;
        let mut next = None;
        let mut byte_idx = pc / 8;

        // search for the bit in the current byte
        let mut search_byte = |byte: u8, start_bit: usize| {
            for bit_idx in start_bit..8 {
                if (byte >> bit_idx) & 1 == 1 {
                    return Some(pc);
                }
                pc += 1;
            }

            None
        };

        // search for the bit in the first byte
        let bit_idx = self.position % 8;
        if bit_idx > 0 {
            next = search_byte(bitmask[byte_idx], bit_idx);
            byte_idx += 1;
        }

        // search for the bit in the rest of the bytes
        while let (Some(byte), None) = (bitmask.get(byte_idx), next) {
            next = search_byte(*byte, 0);
            byte_idx += 1;
        }

        // return the next instruction position, or the end of the buffer
        next.unwrap_or(self.buffer.len()).min(24)
    }
}

/// A wrapped value with an offset range.
pub struct Offset<T> {
    /// The range.
    pub range: Range<usize>,

    /// The value.
    pub value: T,
}
