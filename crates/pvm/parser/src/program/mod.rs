//! Program blob.

use crate::Memory;
use anyhow::Result;
pub use {
    blob::{deblob, ProgramBlob},
    metadata::{ConventionalMetadata, CrateInfo},
    preimage::PreimageBlob,
    standard::{standard, StandardProgramBlob},
};

mod blob;
mod metadata;
mod preimage;
mod standard;

/// Convert a preimage blob to a program.
pub fn preimage(blob: Vec<u8>, args: &[u8]) -> anyhow::Result<Program> {
    let preimage = PreimageBlob::from_bytes(&blob)?;
    preimage.blob.init(args)
}

/// A PVM program.
pub struct Program {
    /// The program code (c).
    pub code: Vec<u8>,

    /// The registers (ω).
    pub registers: [u64; 13],

    /// The memory (µ).
    pub memory: Memory,
}

impl Program {
    pub fn blob(&self) -> Result<ProgramBlob<'_>> {
        crate::deblob(self.code.as_ref())
    }
}
