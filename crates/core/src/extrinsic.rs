pub use {assurance::*, guarantee::*, preimage::*};

// --------------------------------------------
// Preimage types
// --------------------------------------------
mod preimage {
    use crate::misc::*;
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
    use crate::{misc::*, AVAIL_BITFIELD_BYTES};
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

    /// Represents a sequence of assurances.
    pub type AssurancesExtrinsic = Vec<AvailAssurance>;
}

// --------------------------------------------
// Guarantee types
// --------------------------------------------
mod guarantee {
    use crate::misc::*;
    use crate::work::report::*;
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

    /// Represents a sequence of guarantees.
    pub type GuaranteesExtrinsic = Vec<ReportGuarantee>;
}
