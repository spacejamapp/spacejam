//! Program blob.

use crate::{reader::Reader, util};
use anyhow::Result;
use std::collections::BTreeMap;

/// The code section.
///
/// The program blob `p` is split into as series of octets which make
/// up the instruction data `c` and the opcode bitmask `k` as well as
/// the jump table `j`.
///
/// The latter, dynamic jump table, is a sequence of indices into the
/// instruction data blob and is indexed into when dynamically-computed
/// jumps are taken. It is encoded as a sequence of natural numbers
/// (i.e. non-negative integers) each encoded with the same length in
/// octets. This length, term z above, is itself encoded prior.
///
/// `p` = E(∣j∣)⌢ E1(z)⌢ E(∣c∣)⌢ Ez(j)⌢ E(c)⌢ E(k), ∣k∣= ∣c∣
#[derive(Default)]
pub struct ProgramBlob {
    /// The instructions (c).
    pub instructions: Vec<u8>,

    /// The bitmask of the instruction data (k).
    pub bitmask: Vec<u8>,

    /// The jump table (j).
    pub jump_table: Vec<u64>,
}

impl ProgramBlob {
    /// Get the reader.
    pub fn reader(&self) -> Reader<'_> {
        Reader::new(&self.instructions, &self.bitmask)
    }
}

impl TryFrom<&[u8]> for ProgramBlob {
    type Error = anyhow::Error;

    fn try_from(blob: &[u8]) -> Result<Self> {
        util::deblob(blob)
    }
}

/// The standard program blob.
#[derive(Default)]
pub struct StandardProgramBlob {
    /// The program code (c).
    pub code: Vec<u8>,

    /// The registers (ω).
    pub registers: [u64; 13],

    /// The memory (µ).
    pub memory: BTreeMap<u32, (Vec<u8>, bool)>,
}

impl From<workaround::ProgramBlob<'_>> for StandardProgramBlob {
    fn from(blob: workaround::ProgramBlob<'_>) -> Self {
        tracing::trace!("converting jam_program_blob::ProgramBlob to StandardProgramBlob");
        tracing::trace!("ro data length: {}", blob.ro_data.len());
        tracing::trace!("rw data length: {}", blob.rw_data.len());
        tracing::trace!("code length: {}", blob.code_blob.len());
        tracing::trace!("rw data padding pages: {:?}", blob.rw_data_padding_pages);
        tracing::trace!("stack size: {}", blob.stack_size);

        // Extract data from the workaround format
        let ro_data = blob.ro_data.to_vec();
        let rw_data = blob.rw_data.to_vec();
        let code = blob.code_blob.to_vec();
        let z = blob.rw_data_padding_pages as u64; // padding pages
        let s = blob.stack_size as u64; // stack size

        // Encode in the standard format: E₃(|o|) ⌢ E₃(|w|) ⌢ E₂(z) ⌢ E₃(s) ⌢ o ⌢ w ⌢ E₄(|c|) ⌢ c
        let mut standard_blob = Vec::new();

        // E₃(|o|) - encode ro_data length as 3 bytes (little-endian)
        let ro_len = ro_data.len() as u64;
        standard_blob.extend_from_slice(&ro_len.to_le_bytes()[..3]);

        // E₃(|w|) - encode rw_data length as 3 bytes (little-endian)
        let rw_len = rw_data.len() as u64;
        standard_blob.extend_from_slice(&rw_len.to_le_bytes()[..3]);

        // E₂(z) - encode padding pages as 2 bytes (little-endian)
        standard_blob.extend_from_slice(&z.to_le_bytes()[..2]);

        // E₃(s) - encode stack size as 3 bytes (little-endian)
        standard_blob.extend_from_slice(&s.to_le_bytes()[..3]);

        // o - ro_data
        standard_blob.extend_from_slice(&ro_data);

        // w - rw_data
        standard_blob.extend_from_slice(&rw_data);

        // E₄(|c|) - encode code length as 4 bytes (little-endian)
        let code_len = code.len() as u64;
        standard_blob.extend_from_slice(&code_len.to_le_bytes()[..4]);

        // c - code
        standard_blob.extend_from_slice(&code);

        // Parse using the standard function (no args for this format)
        crate::util::standard(&standard_blob, &[]).unwrap_or_else(|e| {
            tracing::error!("Failed to parse standard format: {}", e);
            // Fallback to empty standard program blob
            StandardProgramBlob::default()
        })
    }
}

pub mod workaround {
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
    pub struct ProgramBlob<'a> {
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

    impl<'a> ProgramBlob<'a> {
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

            Some(ProgramBlob {
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
}
