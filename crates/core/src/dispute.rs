//! Dispute types

use crate::{misc::*, VALIDATORS_SUPER_MAJORITY};
use codec::Json;
use serde::{Deserialize, Serialize};

/// Represents a judgement in a dispute.
#[derive(Debug, Serialize, Deserialize, Json, Copy, Clone, PartialEq, Eq)]
pub struct Judgement {
    pub vote: bool,
    pub index: ValidatorIndex,
    #[json(hex)]
    #[serde(with = "codec")]
    pub signature: Ed25519Signature,
}

impl Default for Judgement {
    fn default() -> Self {
        Judgement {
            vote: false,
            index: 0,
            signature: [0u8; 64],
        }
    }
}

/// Represents a verdict in a dispute.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct Verdict {
    #[json(hex)]
    pub target: OpaqueHash,
    pub age: u32,
    #[json(Vec<JudgementJson>)]
    pub votes: [Judgement; VALIDATORS_SUPER_MAJORITY as usize],
}

/// Represents a culprit in a dispute.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct Culprit {
    #[json(hex)]
    pub target: OpaqueHash,
    #[json(hex)]
    pub key: Ed25519Public,
    #[json(hex)]
    #[serde(with = "codec")]
    pub signature: Ed25519Signature,
}

/// Represents a fault in a dispute.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct Fault {
    #[json(hex)]
    pub target: OpaqueHash,
    pub vote: bool,
    #[json(hex)]
    pub key: Ed25519Public,
    #[json(hex)]
    #[serde(with = "codec")]
    pub signature: Ed25519Signature,
}

/// Represents the records of disputes.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct DisputesRecords {
    /// [ψ_g] Good records
    #[json(hex)]
    pub good: Vec<OpaqueHash>,
    /// [ψ_b] Bad records
    #[json(hex)]
    pub bad: Vec<OpaqueHash>,
    /// [ψ_w] Wonky records
    #[json(hex)]
    pub wonky: Vec<OpaqueHash>,
    /// [ψ_o] Offenders
    #[json(hex)]
    pub offenders: Vec<Ed25519Public>,
}

/// Represents the extrinsic data for disputes.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct DisputesExtrinsic {
    /// [ψ_v] Verdicts
    #[json(nested)]
    pub verdicts: Vec<Verdict>,
    /// [ψ_c] Culprits
    #[json(nested)]
    pub culprits: Vec<Culprit>,
    /// [ψ_f] Faults
    #[json(nested)]
    pub faults: Vec<Fault>,
}
