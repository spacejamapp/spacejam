//! Program blob.

use codec::JamCodec;
use std::{borrow::Cow, collections::BTreeMap, ops::Range};
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
pub fn preimage<'a>(blob: &'a [u8], args: &'a [u8]) -> anyhow::Result<Program<'a>> {
    let preimage = PreimageBlob::from_bytes(blob)?;
    let metadata = ConventionalMetadata::decode(&preimage.metadata)?;
    tracing::debug!("metadata: {metadata:?}");
    preimage.blob.init(args)
}

/// A PVM program.
pub struct Program<'a> {
    /// The program code (c).
    pub code: Cow<'a, [u8]>,

    /// The registers (ω).
    pub registers: [u64; 13],

    /// The memory (µ).
    pub memory: BTreeMap<u32, (Vec<u8>, bool)>,

    /// The initial heap pointer.
    pub heap: Range<u32>,
}

/// (µ) The memory of a program.
pub struct Memory<'a> {
    /// The memory (µ).
    pub memory: BTreeMap<u32, (Cow<'a, [u8]>, bool)>,
}
