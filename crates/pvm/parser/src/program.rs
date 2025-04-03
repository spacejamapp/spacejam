//! Program blob.

use crate::reader::Reader;
use anyhow::Result;
use codec::compact::Numeric;
use core::ops::Range;

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
/// `p` = E(∣j∣)⌢ E1(z)⌢ E(∣c∣)⌢ Ez(j)⌢ E(c)⌢ E(k), ∣k∣= ∣c∣
#[derive(Default)]
pub struct ProgramBlob {
    /// The instructions (c).
    pub instructions: Vec<u8>,

    /// The bitmask of the instruction data (k).
    pub bitmask: Vec<u8>,

    /// The jump table (j).
    pub jump_table: Vec<u64>,

    /// The range of the code.
    pub range: Range<usize>,
}

impl ProgramBlob {
    /// Get the reader.
    pub fn reader(&self) -> Reader<'_> {
        Reader::new(&self.instructions, &self.bitmask)
    }
}

impl TryFrom<&[u8]> for ProgramBlob {
    type Error = anyhow::Error;

    fn try_from(blob: &[u8]) -> Result<Self> {
        let mut pos = 0;

        // decode the jump table length
        //
        // E(|j|)
        let (len, next) = codec::compact::decode_from(blob);
        let jump_table_len = len as usize;
        pos += next;

        // decode the jump table entry size
        //
        // E₁(z)
        let jump_table_entry_size = blob[pos] as usize;
        pos += 1;

        // decode the instruction data length
        //
        // E(|c|)
        let (len, next) = codec::compact::decode_from(&blob[pos..]);
        let instruction_len = len as usize;
        pos += next;

        // decode the jump table
        //
        // E_z(j)
        let jump_table = if jump_table_entry_size > 0 {
            let length = jump_table_len * jump_table_entry_size;
            let table = blob[pos..pos + length].to_vec();
            let jump = table
                .chunks(jump_table_entry_size)
                .map(u64::decode)
                .collect();

            pos += length;
            jump
        } else {
            vec![]
        };

        // decode the instruction data
        //
        // E(c)
        let instructions = blob[pos..pos + instruction_len].to_vec();
        pos += instruction_len;

        // decode the bitmask
        //
        // E(k)
        let bitmask = blob[pos..].to_vec();

        Ok(Self {
            jump_table,
            instructions,
            bitmask,
            range: 0..blob.len(),
        })
    }
}
