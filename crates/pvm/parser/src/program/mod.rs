//! Program blob.

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
    pub fn instr_reader<'r>(&'r self) -> InstructionReader<'r> {
        self.instruction_data.reader()
    }
}
