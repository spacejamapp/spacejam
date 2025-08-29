//! Preimage program blob.

use crate::program::StandardProgramBlob;
use codec::{io, Reader, Writer};
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

    /// Convert the preimage blob to a vector of bytes.
    pub fn to_vec(&self) -> Result<Vec<u8>, &'static str> {
        let blob = &self.blob;
        let mut output = Vec::new();
        output.write_var(u32::try_from(self.metadata.len()).map_err(|_| "metadata too large")?);
        output.extend_from_slice(&self.metadata);
        output.write_u24(u32::try_from(blob.ro_data.len()).map_err(|_| "too large RO data")?);
        output.write_u24(u32::try_from(blob.rw_data.len()).map_err(|_| "too large RW data")?);
        output.extend_from_slice(&blob.rw_data_padding_pages.to_le_bytes());
        output.write_u24(blob.stack_size);
        output.extend_from_slice(&blob.ro_data);
        output.extend_from_slice(&blob.rw_data);
        output.write_u32(u32::try_from(blob.code_blob.len()).map_err(|_| "too large code")?);
        output.extend_from_slice(&blob.code_blob);
        Ok(output)
    }
}
