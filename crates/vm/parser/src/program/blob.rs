//! Program blob.
//!
//! TODO: correct the usage of bitmask.

use crate::{
    reader::{Offset, Reader},
    Instruction,
};
use anyhow::Result;
use codec::{compact::Numeric, io, Reader as _};
use std::{borrow::Cow, collections::BTreeMap};

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
pub struct ProgramBlob<'a> {
    /// The instructions (c).
    pub instructions: Cow<'a, [u8]>,

    /// The bitmask of the instruction data (k).
    pub bitmask: Cow<'a, [u8]>,

    /// The jump table (j).
    pub jump_table: Vec<u64>,
}

impl ProgramBlob<'_> {
    /// Get the reader.
    pub fn reader(&self) -> Reader<'_> {
        Reader::new(&self.instructions, &self.bitmask)
    }

    /// Read all instructions.
    pub fn read_all(&self) -> Result<BTreeMap<u64, Offset<Instruction>>> {
        let mut reader = self.reader();
        let mut instructions = BTreeMap::new();
        while let Ok(instr) = reader.read() {
            instructions.insert(instr.range.start as u64, instr);
        }
        Ok(instructions)
    }

    /// Read all blocks.
    ///
    /// TODO: use the bitmask or the introduce dispatch method for this.
    pub fn read_blocks(&self) -> Result<BTreeMap<u64, Vec<Offset<Instruction>>>> {
        let mut reader = self.reader();
        let mut blocks = BTreeMap::new();
        let mut block = Vec::new();
        let mut start = 0;
        while let Ok(instr) = reader.read() {
            let is_termination = instr.value.is_termination();
            block.push(instr);
            if is_termination {
                blocks.insert(start, block);
                block = Vec::new();
                start = reader.position as u64;
            }
        }

        if !block.is_empty() {
            blocks.insert(start, block);
        }
        Ok(blocks)
    }
}

impl<'a> TryFrom<&'a [u8]> for ProgramBlob<'a> {
    type Error = anyhow::Error;

    fn try_from(blob: &'a [u8]) -> Result<Self> {
        self::deblob(blob)
    }
}

/// The `deblob` function.
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
pub fn deblob(mut blob: &[u8]) -> Result<ProgramBlob<'_>> {
    // E(|j|) decode the jump table length
    let jump_table_len = blob
        .read_var()
        .ok_or_else(|| anyhow::anyhow!("EOF while reading jump table length"))?;

    // E₁(z) decode the jump table entry size
    let jump_table_entry_size = blob
        .read_u8()
        .ok_or_else(|| anyhow::anyhow!("EOF while reading jump table entry size"))?;

    // E(|c|) decode the instruction data length
    let instruction_len = blob
        .read_var()
        .ok_or_else(|| anyhow::anyhow!("EOF while reading instruction length"))?;

    // E_z(j) decode the jump table
    let mut jump = vec![];
    if jump_table_entry_size > 0 {
        let length = jump_table_len * jump_table_entry_size as u32;
        let table = io::read_cow(&mut blob, length)
            .ok_or_else(|| anyhow::anyhow!("EOF while reading jump table"))?;
        jump = table
            .chunks(jump_table_entry_size as usize)
            .map(u64::decode)
            .collect();
    }

    // E(c) decode the instruction data
    let instructions = io::read_cow(&mut blob, instruction_len)
        .ok_or_else(|| anyhow::anyhow!("EOF while reading instruction data"))?;

    // check that the program blob is not empty
    if instructions.is_empty() {
        anyhow::bail!("empty program blob");
    }

    // E(k) decode the bitmask
    let len = blob.len();
    let bitmask = io::read_cow(&mut blob, len as u32)
        .ok_or_else(|| anyhow::anyhow!("EOF while reading bitmask"))?;

    // TODO: bitmask length check
    //
    // if bitmask.len() * 8 != instructions.len() {
    //     return Err("bitmask length does not match instruction length");
    // }

    Ok(ProgramBlob {
        instructions,
        bitmask,
        jump_table: jump,
    })
}
