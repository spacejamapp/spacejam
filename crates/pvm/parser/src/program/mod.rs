//! Program blob.

pub use {
    blob::{deblob, ProgramBlob},
    preimage::PreimageBlob,
    standard::{standard, StandardProgramBlob},
};

pub mod blob;
pub mod preimage;
pub mod standard;

/// Convert a preimage blob to a standard program blob.
pub fn to_standard(blob: &[u8]) -> anyhow::Result<StandardProgramBlob> {
    let program = PreimageBlob::from_bytes(blob).ok_or(anyhow::anyhow!("Invalid preimage blob"))?;
    Ok(program.into())
}
