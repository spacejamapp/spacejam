use crate::{Ed25519Signature, OpaqueHash, ValidatorIndex, AVAIL_BITFIELD_BYTES, CORES_COUNT};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents an assurance of availability.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct AvailAssurance {
    /// The anchor of the assurance.
    #[json(hex)]
    pub anchor: OpaqueHash,

    /// The bitfield of the assurance.
    pub bitfield: [u8; AVAIL_BITFIELD_BYTES],

    /// The index of the validator that signed the assurance.
    pub validator_index: ValidatorIndex,

    /// The signature of the assurance.
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub signature: Ed25519Signature,
}

impl AvailAssurance {
    /// Returns the bitsmap of the assurance.
    pub fn bitsmap(&self) -> [u8; CORES_COUNT] {
        let mut bitsmap = [0u8; CORES_COUNT];
        for (core_idx, bit) in bitsmap.iter_mut().enumerate() {
            *bit = (self.bitfield[core_idx / 8] >> (core_idx % 8)) & 1;
        }
        bitsmap
    }

    #[cfg(feature = "crypto")]
    /// Returns the message that was signed by the assurance.
    ///
    /// reference graypapar 11.2.1
    pub fn singing_message(&self) -> Vec<u8> {
        let mut message = vec![];
        message.extend_from_slice(&crate::JAM_AVAILABLE);

        let hashed = crypto::blake2b(&[self.anchor.to_vec(), self.bitfield.to_vec()].concat());
        message.extend_from_slice(&hashed);
        message
    }
}

/// Represents a sequence of assurances.
pub type AssurancesExtrinsic = Vec<AvailAssurance>;
