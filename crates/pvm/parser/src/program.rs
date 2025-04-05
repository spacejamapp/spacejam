//! Program blob.

use crate::{reader::Reader, util};
use anyhow::Result;
use std::collections::BTreeMap;

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
        util::deblob(blob)
    }
}

/// The standard program blob.
#[derive(Default)]
pub struct StandardProgramBlob {
    /// The program code (c).
    pub code: Vec<u8>,

    /// The registers (ω).
    pub registers: [u64; 13],

    /// The memory (µ).
    pub memory: BTreeMap<u32, (Vec<u8>, bool)>,
}

impl TryFrom<&[u8]> for StandardProgramBlob {
    type Error = anyhow::Error;

    fn try_from(blob: &[u8]) -> Result<Self> {
        util::init(blob)
    }
}

/// The memory trait.
pub trait Memory: Default + Clone {
    /// Create a new mwemory from a raw memory.
    fn from_raw(memory: BTreeMap<u32, (Vec<u8>, bool)>) -> Self;

    /// Check if the memory contains the given data.
    fn contains(&self, data: &[u8]) -> bool;
}

impl Memory for () {
    fn from_raw(_memory: BTreeMap<u32, (Vec<u8>, bool)>) -> Self {}

    fn contains(&self, _data: &[u8]) -> bool {
        false
    }
}
