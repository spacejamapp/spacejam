//! Preimage program blob.

use codec::{io, Reader, Writer};
use std::{borrow::Cow, string::String, vec::Vec};

/// Information on a crate, useful for building conventional metadata of type 0.
#[derive(Clone, PartialEq, Eq)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub license: String,
    pub authors: Vec<String>,
}

/// Information which, when encoded, could fill a program blob's metadata.
#[derive(Clone, PartialEq, Eq)]
pub enum ConventionalMetadata {
    Info(CrateInfo),
}

/// A JAM-specific program blob.
pub struct PreimageBlob<'a> {
    pub metadata: Cow<'a, [u8]>,
    pub ro_data: Cow<'a, [u8]>,
    pub rw_data: Cow<'a, [u8]>,
    pub code_blob: Cow<'a, [u8]>,
    pub rw_data_padding_pages: u16,
    pub stack_size: u32,
}

impl<'a> PreimageBlob<'a> {
    pub fn from_bytes(mut bytes: &'a [u8]) -> Option<Self> {
        let metadata_len = bytes.read_var()?;
        let metadata = io::read_cow(&mut bytes, metadata_len)?;
        let ro_data_len = bytes.read_u24()?;
        let rw_data_len = bytes.read_u24()?;
        let rw_data_padding_pages = bytes.read_u16()?;
        let stack_size = bytes.read_u24()?;
        let ro_data = io::read_cow(&mut bytes, ro_data_len)?;
        let rw_data = io::read_cow(&mut bytes, rw_data_len)?;
        let code_blob_len = bytes.read_u32()?;
        let code_blob = io::read_cow(&mut bytes, code_blob_len)?;

        Some(PreimageBlob {
            metadata,
            rw_data_padding_pages,
            stack_size,
            ro_data,
            rw_data,
            code_blob,
        })
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, &'static str> {
        let mut output = Vec::new();
        output.write_var(u32::try_from(self.metadata.len()).map_err(|_| "metadata too large")?);
        output.extend_from_slice(&self.metadata);
        output.write_u24(u32::try_from(self.ro_data.len()).map_err(|_| "too large RO data")?);
        output.write_u24(u32::try_from(self.rw_data.len()).map_err(|_| "too large RW data")?);
        output.extend_from_slice(&self.rw_data_padding_pages.to_le_bytes());
        output.write_u24(self.stack_size);
        output.extend_from_slice(&self.ro_data);
        output.extend_from_slice(&self.rw_data);
        output.write_u32(u32::try_from(self.code_blob.len()).map_err(|_| "too large code")?);
        output.extend_from_slice(&self.code_blob);
        Ok(output)
    }
}
