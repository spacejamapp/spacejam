//! Misc types

pub use {core::*, crypto::*, service::*, stats::*};

// --------------------------------------------
// crypto types
// --------------------------------------------
mod crypto {
    pub type BandersnatchPublic = [u8; 32];
    pub type Ed25519Public = [u8; 32];
    pub type BlsPublic = [u8; 144];
    pub type BandersnatchVrfSignature = [u8; 96];
    pub type BandersnatchRingCommitment = [u8; 144];
    pub type BandersnatchRingVrfSignature = [u8; 784];
    pub type Ed25519Signature = [u8; 64];
}

// --------------------------------------------
// application specific core types
// --------------------------------------------
mod core {
    use crate::{extrinsic::AvailAssurance, misc::crypto::*};
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    pub type OpaqueHash = [u8; 32];
    pub type TimeSlot = u32;
    pub type ValidatorIndex = u16;
    pub type CoreIndex = u16;

    pub type HeaderHash = OpaqueHash;
    pub type StateRoot = OpaqueHash;
    pub type BeefyRoot = OpaqueHash;
    pub type WorkPackageHash = OpaqueHash;
    pub type WorkReportHash = OpaqueHash;
    pub type ExportsRoot = OpaqueHash;
    pub type ErasureRoot = OpaqueHash;

    pub type Gas = u64;
    pub type Entropy = OpaqueHash;
    pub type EntropyBuffer = [Entropy; 4];
    pub type ValidatorMetadata = [u8; 128];

    /// Represents the ValidatorData structure from ASN.1
    #[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
    pub struct ValidatorData {
        #[json(hex)]
        pub bandersnatch: BandersnatchPublic,
        #[json(hex)]
        pub ed25519: Ed25519Public,
        #[json(hex)]
        #[serde(with = "codec::bytes")]
        pub bls: BlsPublic,
        #[json(hex)]
        #[serde(with = "codec::bytes")]
        pub metadata: ValidatorMetadata,
    }

    impl ValidatorData {
        /// Returns the bitsmap of the validator.
        pub fn verify_assurance(&self, assurance: &AvailAssurance) -> anyhow::Result<()> {
            crypto::ed25519::verify(
                &assurance.singing_message(),
                assurance.signature,
                self.ed25519,
            )?;
            Ok(())
        }
    }

    impl Default for ValidatorData {
        fn default() -> Self {
            ValidatorData {
                bandersnatch: Default::default(),
                ed25519: Default::default(),
                bls: [0; 144],
                metadata: [0; 128],
            }
        }
    }

    pub type ValidatorsData = Vec<ValidatorData>;

    /// Represents the RefineContext structure from ASN.1
    #[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
    pub struct RefineContext {
        #[json(hex)]
        pub anchor: HeaderHash,
        #[json(hex)]
        pub state_root: StateRoot,
        #[json(hex)]
        pub beefy_root: BeefyRoot,
        #[json(hex)]
        pub lookup_anchor: HeaderHash,
        pub lookup_anchor_slot: TimeSlot,
        #[json(hex)]
        pub prerequisites: Vec<OpaqueHash>,
    }
}

// --------------------------------------------
// Service types
// --------------------------------------------
mod service {
    use super::core::*;
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    pub type ServiceId = u32;

    /// Represents a service info.
    #[derive(Debug, Serialize, Deserialize, Json, Clone, PartialEq, Eq)]
    pub struct ServiceInfo {
        #[json(hex)]
        pub code_hash: OpaqueHash,
        pub balance: u64,
        pub min_item_gas: Gas,
        pub min_memo_gas: Gas,
        pub bytes: u64,
        pub items: u32,
    }
}

// --------------------------------------------
// Stats types
// --------------------------------------------
mod stats {
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    /// Represents an activity record.
    #[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
    pub struct ActivityRecord {
        pub blocks: u32,
        pub tickets: u32,
        pub pre_images: u32,
        pub pre_images_size: u32,
        pub guarantees: u32,
        pub assurances: u32,
    }

    /// Represents statistics.
    #[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
    pub struct Statistics {
        #[json(nested)]
        pub current: Vec<ActivityRecord>,
        #[json(nested)]
        pub last: Vec<ActivityRecord>,
    }
}
