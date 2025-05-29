//! Program blob.

use std::{borrow::Cow, collections::BTreeMap};
pub use {
    blob::{deblob, ProgramBlob},
    preimage::PreimageBlob,
    standard::{standard, StandardProgramBlob},
};

mod blob;
mod preimage;
mod standard;

/// A PVM program.
pub struct Program<'a> {
    /// The program code (c).
    pub code: Cow<'a, [u8]>,

    /// The registers (ω).
    pub registers: [u64; 13],

    /// The memory (µ).
    pub memory: BTreeMap<u32, (Vec<u8>, bool)>,
}

/// Convert a preimage blob to a program.
pub fn preimage<'a>(blob: &'a [u8], args: &'a [u8]) -> anyhow::Result<Program<'a>> {
    PreimageBlob::from_bytes(blob)?.blob.init(args)
}
