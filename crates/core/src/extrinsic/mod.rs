pub use {assurance::*, dispute::*, guarantee::*, preimage::*, ticket::*};

pub mod dispute;
pub mod ticket;

// --------------------------------------------
// Preimage types
// --------------------------------------------
mod preimage {
    use crate::service::ServiceId;
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    /// Represents a preimage request.
    #[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct Preimage {
        pub requester: ServiceId,
        #[json(hex)]
        pub blob: Vec<u8>,
    }

    /// Represents a sequence of preimages.
    pub type PreimagesExtrinsic = Vec<Preimage>;
}

// --------------------------------------------
// Assurance types
// --------------------------------------------
mod assurance {
    use crate::{
        Ed25519Signature, OpaqueHash, ValidatorIndex, AVAIL_BITFIELD_BYTES, CORES_COUNT,
        JAM_AVAILABLE,
    };
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    /// Represents an assurance of availability.
    #[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
    pub struct AvailAssurance {
        #[json(hex)]
        pub anchor: OpaqueHash,
        pub bitfield: [u8; AVAIL_BITFIELD_BYTES],
        pub validator_index: ValidatorIndex,
        #[json(hex)]
        #[serde(with = "codec::bytes")]
        pub signature: Ed25519Signature,
    }

    impl AvailAssurance {
        /// Returns the bitsmap of the assurance.
        pub fn bitsmap(&self) -> [u8; CORES_COUNT] {
            let mut bitsmap = [0u8; CORES_COUNT];
            for (core_idx, bit) in bitsmap.iter_mut().enumerate() {
                *bit = self.bitfield[core_idx / 8] >> (core_idx % 8) & 1;
            }
            bitsmap
        }

        /// Returns the message that was signed by the assurance.
        ///
        /// reference graypapar 11.2.1
        pub fn singing_message(&self) -> Vec<u8> {
            let mut message = vec![];
            message.extend_from_slice(&JAM_AVAILABLE);

            let hashed = crypto::blake2b(&[self.anchor.to_vec(), self.bitfield.to_vec()].concat());
            message.extend_from_slice(&hashed);
            message
        }
    }

    /// Represents a sequence of assurances.
    pub type AssurancesExtrinsic = Vec<AvailAssurance>;
}

// --------------------------------------------
// Guarantee types
// --------------------------------------------
mod guarantee {
    use crate::{work::report::*, Ed25519Signature, TimeSlot, ValidatorIndex, JAM_GUARANTEE};
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    /// Represents a signature from a validator.
    #[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
    pub struct ValidatorSignature {
        pub validator_index: ValidatorIndex,
        #[json(hex)]
        #[serde(with = "codec::bytes")]
        pub signature: Ed25519Signature,
    }

    /// Represents a report guarantee.
    #[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
    pub struct ReportGuarantee {
        #[json(nested)]
        pub report: WorkReport,
        pub slot: TimeSlot,
        #[json(nested)]
        pub signatures: Vec<ValidatorSignature>,
    }

    impl ReportGuarantee {
        /// Returns the message that was signed by the guarantors.
        pub fn signing_message(&self) -> anyhow::Result<Vec<u8>> {
            let mut message = vec![];
            message.extend_from_slice(&JAM_GUARANTEE);

            let hashed = crypto::blake2b(&codec::encode(&self.report)?);
            message.extend_from_slice(&hashed);
            Ok(message)
        }
    }

    /// Represents a sequence of guarantees.
    pub type GuaranteesExtrinsic = Vec<ReportGuarantee>;
}
