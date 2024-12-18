//! Program blob.

use anyhow::Result;
use core::ops::Range;
pub use {
    data::{InstructionData, InstructionReader},
    jump::JumpTable,
};

mod data;
mod jump;

/// The code section.
pub struct ProgramBlob {
    /// The jump table.
    pub jump_table: JumpTable,

    /// The instruction data.
    pub instruction_data: InstructionData,

    /// The range of the code.
    pub range: Range<usize>,
}

impl ProgramBlob {
    /// Get the instruction reader.
    pub fn instr_reader(&self) -> InstructionReader<'_> {
        self.instruction_data.reader()
    }
}

impl TryFrom<&[u8]> for ProgramBlob {
    type Error = anyhow::Error;

    fn try_from(blob: &[u8]) -> Result<Self> {
        let jump_table_len = &blob[0..2];
        if jump_table_len != [0, 0] {
            anyhow::bail!("does not support jump tables atm");
        }

        // FIXME: only support 1 byte instruction data length for now
        let instruction_len = blob[2] as usize;
        let instruction_data = InstructionData {
            instructions: blob[3..instruction_len + 3].to_vec(),
            bitmask: blob[instruction_len + 3..].to_vec(),
            range: 3..blob.len(),
        };

        Ok(Self {
            jump_table: JumpTable::default(),
            instruction_data,
            range: 0..blob.len(),
        })
    }
}
