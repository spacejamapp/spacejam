//! Preimage program blob.

use jam_codec::{Compact, Decode, Encode};
use std::{borrow::Cow, string::String, vec::Vec};

/// Information on a crate, useful for building conventional metadata of type 0.
#[derive(Clone, PartialEq, Eq, Encode, Decode)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub license: String,
    pub authors: Vec<String>,
}

/// Information which, when encoded, could fill a program blob's metadata.
#[derive(Clone, PartialEq, Eq, Encode, Decode)]
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

fn read_u24(bytes: &mut &[u8]) -> Option<u32> {
    let xs = bytes.get(..3)?;
    *bytes = &bytes[3..];
    Some(u32::from_le_bytes([xs[0], xs[1], xs[2], 0]))
}

fn write_u24(value: u32, output: &mut Vec<u8>) -> Result<(), ()> {
    if value >= (1 << 24) {
        return Err(());
    }

    output.extend_from_slice(&value.to_le_bytes()[0..3]);
    Ok(())
}

fn read_u16(bytes: &mut &[u8]) -> Option<u16> {
    let xs = bytes.get(..2)?;
    *bytes = &bytes[2..];
    Some(u16::from_le_bytes([xs[0], xs[1]]))
}

fn read_u32(bytes: &mut &[u8]) -> Option<u32> {
    let xs = bytes.get(..4)?;
    *bytes = &bytes[4..];
    Some(u32::from_le_bytes([xs[0], xs[1], xs[2], xs[3]]))
}

fn read_var(bytes: &mut &[u8]) -> Option<u32> {
    Some(Compact::<u32>::decode(bytes).ok()?.0)
}

fn write_var(value: u32, output: &mut Vec<u8>) {
    Compact::<u32>(value).encode_to(output)
}

fn read_cow<'a>(bytes: &mut &'a [u8], length: u32) -> Option<Cow<'a, [u8]>> {
    let length = length as usize;
    let cow = bytes.get(..length)?;
    *bytes = &bytes[length..];
    Some(Cow::Borrowed(cow))
}

impl<'a> PreimageBlob<'a> {
    pub fn from_bytes(mut bytes: &'a [u8]) -> Option<Self> {
        let offset = read_var(&mut bytes)?;
        let metadata = read_cow(&mut bytes, offset)?;
        let ro_data_len = read_u24(&mut bytes)?;
        let rw_data_len = read_u24(&mut bytes)?;
        let rw_data_padding_pages = read_u16(&mut bytes)?;
        let stack_size = read_u24(&mut bytes)?;
        let ro_data = read_cow(&mut bytes, ro_data_len)?;
        let rw_data = read_cow(&mut bytes, rw_data_len)?;
        let code_blob_len = read_u32(&mut bytes)?;
        let code_blob = read_cow(&mut bytes, code_blob_len)?;

        if !bytes.is_empty() {
            return None;
        }

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
        write_var(
            u32::try_from(self.metadata.len()).map_err(|_| "metadata too large")?,
            &mut output,
        );
        output.extend_from_slice(&self.metadata);
        write_u24(
            u32::try_from(self.ro_data.len()).map_err(|_| "too large RO data")?,
            &mut output,
        )
        .map_err(|_| "too large RO data")?;
        write_u24(
            u32::try_from(self.rw_data.len()).map_err(|_| "too large RW data")?,
            &mut output,
        )
        .map_err(|_| "too large RW data")?;
        output.extend_from_slice(&self.rw_data_padding_pages.to_le_bytes());
        write_u24(self.stack_size, &mut output).map_err(|_| "too large stack size")?;
        output.extend_from_slice(&self.ro_data);
        output.extend_from_slice(&self.rw_data);
        output.extend_from_slice(
            &u32::try_from(self.code_blob.len())
                .map_err(|_| "too large code")?
                .to_le_bytes(),
        );
        output.extend_from_slice(&self.code_blob);
        Ok(output)
    }
}
