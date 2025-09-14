//! Preimage program blob.

use crate::program::StandardProgramBlob;
use codec::{io, Reader};
use std::vec::Vec;

/// A JAM-specific program blob.
pub struct PreimageBlob {
    /// the program metadata
    pub metadata: Vec<u8>,

    /// The standard program blob
    pub blob: StandardProgramBlob,
}

impl PreimageBlob {
    /// Convert a preimage blob to a vector of bytes.
    pub fn from_bytes(mut bytes: &[u8]) -> anyhow::Result<Self> {
        let metadata_len = bytes
            .read_var()
            .ok_or_else(|| anyhow::anyhow!("EOF while reading metadata length"))?;
        let metadata = io::read_cow(&mut bytes, metadata_len)
            .ok_or_else(|| anyhow::anyhow!("EOF while reading metadata"))?;
        let blob = StandardProgramBlob::try_from(bytes)?;
        Ok(PreimageBlob {
            metadata: metadata.to_vec(),
            blob,
        })
    }
}
