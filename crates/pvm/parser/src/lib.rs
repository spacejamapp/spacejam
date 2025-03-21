//! The PVM parser.

use anyhow::Result;
pub use {instruction::Instruction, opcode::Opcode, program::ProgramBlob, visitor::Visitor};

pub mod format;
pub mod instruction;
pub mod opcode;
pub mod program;
pub mod reader;
pub mod visitor;

/// The type of the registers.
pub type Register = u64;

/// Parse a PVM program blob.
pub fn parse(blob: Vec<u8>) -> Result<ProgramBlob> {
    ProgramBlob::try_from(blob.as_ref())
}
