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

/// Convert a preimage blob to a program.
pub fn preimage<'a>(blob: &'a [u8], args: &'a [u8]) -> anyhow::Result<Program<'a>> {
    PreimageBlob::from_bytes(blob)?.blob.init(args)
}

/// A PVM program.
pub struct Program<'a> {
    /// The program code (c).
    pub code: Cow<'a, [u8]>,

    /// The registers (ω).
    pub registers: [u64; 13],

    /// The memory (µ).
    pub memory: BTreeMap<u32, (Cow<'a, [u8]>, bool)>,
}

/// (µ) The memory of a program.
pub struct Memory<'a> {
    /// The memory (µ).
    pub memory: BTreeMap<u32, (Cow<'a, [u8]>, bool)>,
}

impl<'a> Memory<'a> {
    
}
