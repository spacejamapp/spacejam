//! Misc types

pub use assurance::*; // Import assurance types
pub use availability::*;
pub use core::*;
pub use crypto::*;
pub use guarantee::*;
pub use preimage::*; // Import preimage types
pub use service::*; // Import guarantee types

// --------------------------------------------
// crypto types
// --------------------------------------------
mod crypto {
    pub type BandersnatchPublic = [u8; 32];
    pub type Ed25519Public = [u8; 32];
    pub type BlsPublic = [u8; 144];
    pub type BandersnatchVrfSignature = [u8; 96];
    pub type BandersnatchRingVrfSignature = [u8; 784];
    pub type Ed25519Signature = [u8; 64];
}

// --------------------------------------------
// application specific core types
// --------------------------------------------
mod core {
    use super::crypto::*;
    use json::Json;
    use scale::{Decode, Encode};

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
    pub struct ValidatorData {
        pub bandersnatch: BandersnatchPublic,
        pub ed25519: Ed25519Public,
        pub bls: BlsPublic,
        pub metadata: ValidatorMetadata,
    }

    pub type ValidatorsData = Vec<ValidatorData>;

    /// Represents the RefineContext structure from ASN.1
    #[derive(Debug, Encode, Decode, Json)]
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
    use json::Json;
    use scale::{Decode, Encode};

    pub type ServiceId = u32;

    /// Represents a service info.
    #[derive(Debug, Encode, Decode, Json)]
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
// Availability types
// --------------------------------------------
mod availability {
    use crate::work::report::*;

    pub type AvailabilityAssignment = (WorkReport, u32);
    pub type AvailabilityAssignmentsItem = Option<AvailabilityAssignment>;
    pub type AvailabilityAssignments = Vec<AvailabilityAssignmentsItem>;
}

// --------------------------------------------
// Preimage types
// --------------------------------------------
mod preimage {
    use super::service::*;
    use json::Json;
    use scale::{Decode, Encode};

    /// Represents a preimage request.
    #[derive(Debug, Encode, Decode, Json)]
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
    use super::core::*;
    use super::crypto::*;
    use json::Json;
    use scale::{Decode, Encode};

    /// Represents an assurance of availability.
    #[derive(Debug, Encode, Decode, Json)]
    pub struct AvailAssurance {
        #[json(hex)]
        pub anchor: OpaqueHash,
        #[json(hex)]
        pub bitfield: Vec<u8>,
        pub validator_index: ValidatorIndex,
        #[json(hex)]
        pub signature: Ed25519Signature,
    }

    /// Represents a sequence of assurances.
    pub type AssurancesExtrinsic = Vec<AvailAssurance>;
}

// --------------------------------------------
// Guarantee types
// --------------------------------------------
mod guarantee {
    use super::core::*;
    use super::crypto::*;
    use crate::work::report::*;
    use json::Json;
    use scale::{Decode, Encode};

    /// Represents a signature from a validator.
    #[derive(Debug, Encode, Decode, Json)]
    pub struct ValidatorSignature {
        pub validator_index: ValidatorIndex,
        #[json(hex)]
        pub signature: Ed25519Signature,
    }

    /// Represents a report guarantee.
    #[derive(Debug, Encode, Decode, Json)]
    pub struct ReportGuarantee {
        #[json(nested)]
        pub report: WorkReport,
        pub slot: TimeSlot,
        #[json(nested)]
        pub signatures: Vec<ValidatorSignature>,
    }

    /// Represents a sequence of guarantees.
    pub type GuaranteesExtrinsic = Vec<ReportGuarantee>;
}
