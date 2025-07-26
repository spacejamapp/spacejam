//! Extrinsic specification for a work item

use score::service::ExtrinsicSpec;

/// Extrinsic specification for a work item
///
/// Contains both the commitment (hash, length) that goes into the work item
/// and the actual data that must be available to guarantors per Gray Paper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extrinsic {
    /// The hash commitment
    pub hash: score::OpaqueHash,
    /// The length of the data
    pub len: u32,
    /// The actual extrinsic data (available to guarantors)
    pub data: Vec<u8>,
}

impl Extrinsic {
    /// Create an ExtrinsicSpec from raw data
    ///
    /// Automatically computes the hash and length according to Gray Paper spec.
    pub fn from_data(data: Vec<u8>) -> Self {
        let hash = crypto::blake2b(&data);
        let len = data.len() as u32;
        Self { hash, len, data }
    }

    /// Create an Extrinsic from hash and length (for reference-only cases)
    ///
    /// Note: This creates an empty data field. Use only when the actual data
    /// is managed separately.
    pub fn from_commitment(hash: score::OpaqueHash, len: u32) -> Self {
        Self {
            hash,
            len,
            data: Vec::new(),
        }
    }

    /// Convert to ExtrinsicSpec
    pub fn spec(&self) -> ExtrinsicSpec {
        ExtrinsicSpec {
            hash: self.hash,
            len: self.len,
        }
    }
}
