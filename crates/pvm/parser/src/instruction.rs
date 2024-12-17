//! The PVM instructions.

use crate::{format::*, opcode::Opcode};

include!(concat!(env!("OUT_DIR"), "/instruction.rs"));

impl Instruction {
    /// Read the instruction from the bytes.
    ///
    /// Returns the instruction and the number of bytes read.
    pub fn read(bytes: &[u8]) -> anyhow::Result<(Self, usize)> {
        let _opcode = Opcode::try_from(bytes[0])?;

        todo!();
    }
}
