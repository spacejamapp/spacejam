//! Program blob.

use crate::reader::{InstructionReader, Reader};
use anyhow::Result;
use core::ops::Range;

pub use jump::JumpTable;

// mod data;
// mod deblob;
mod jump;

/// The code section.
///
/// The program blob `p` is split into as series of octets which make
/// up the instruction data `c` and the opcode bitmask `k` as well as
/// the jump table `j`.
///
/// The latter, dynamic jump table, is a sequence of indices into the
/// instruction data blob and is indexed into when dynamically-computed
/// jumps are taken. It is encoded as a sequence of natural numbers
/// (i.e. non-negative integers) each encoded with the same length in
/// octets. This length, term z above, is itself encoded prior.
///
/// `p` = E(∣j∣)⌢ E1(z)⌢ E(∣c∣)⌢ Ez (j)⌢ E(c)⌢ E(k), ∣k∣= ∣c∣
#[derive(Default)]
pub struct ProgramBlob {
    /// The jump table.
    pub jump_table: JumpTable,

    /// The instructions.
    pub instructions: Vec<u8>,

    /// The bitmask of the instruction data.
    pub bitmask: Vec<u8>,

    /// The range of the code.
    pub range: Range<usize>,
}

impl ProgramBlob {
    /// Get the instruction reader.
    pub fn instr_reader(&self) -> InstructionReader<'_> {
        InstructionReader {
            bitmask: &self.bitmask,
            reader: Reader::new(&self.instructions, self.range.start),
        }
    }

    /// Get the instruction reader at the program counter.
    pub fn instr_reader_at(&self, pc: usize) -> InstructionReader<'_> {
        self.instr_reader().with_position(pc)
    }
}

impl TryFrom<&[u8]> for ProgramBlob {
    type Error = anyhow::Error;

    fn try_from(blob: &[u8]) -> Result<Self> {
        let jump_table_len = &blob[0..2];
        if jump_table_len != [0, 0] {
            println!("jump table length: {:?}", jump_table_len);
            anyhow::bail!("does not support jump tables atm");
        }

        // FIXME: only support 1 byte instruction data length for now
        let instruction_len = blob[2] as usize;
        Ok(Self {
            jump_table: JumpTable::default(),
            instructions: blob[3..instruction_len + 3].to_vec(),
            bitmask: blob[instruction_len + 3..].to_vec(),
            range: 0..blob.len(),
        })
    }
}
