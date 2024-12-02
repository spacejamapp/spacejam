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
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct Verdict {
    #[json(hex)]
    pub target: OpaqueHash,
    pub age: u32,
    #[json(Vec<JudgementJson>)]
    pub votes: [Judgement; VALIDATORS_SUPER_MAJORITY as usize],
}

/// Represents a culprit in a dispute.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
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
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
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
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct DisputesRecords {
    #[json(hex)]
    pub good: Vec<OpaqueHash>,
    #[json(hex)]
    pub bad: Vec<OpaqueHash>,
    #[json(hex)]
    pub wonky: Vec<OpaqueHash>,
    #[json(hex)]
    pub offenders: Vec<Ed25519Public>,
}

/// Represents the extrinsic data for disputes.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct DisputesExtrinsic {
    #[json(nested)]
    pub verdicts: Vec<Verdict>,
    #[json(nested)]
    pub culprits: Vec<Culprit>,
    #[json(nested)]
    pub faults: Vec<Fault>,
}
