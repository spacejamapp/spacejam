//! Dispute types

use crate::misc::*;
use json::Json;
use scale::{Decode, Encode};

/// Represents a judgement in a dispute.
#[derive(Debug, Encode, Decode, Json)]
pub struct Judgement {
    pub vote: bool,
    pub index: ValidatorIndex,
    #[json(hex)]
    pub signature: Ed25519Signature,
}

/// Represents a verdict in a dispute.
#[derive(Debug, Encode, Decode, Json)]
pub struct Verdict {
    #[json(hex)]
    pub target: OpaqueHash,
    pub age: u32,
    #[json(nested)]
    pub votes: Vec<Judgement>,
}

/// Represents a culprit in a dispute.
#[derive(Debug, Encode, Decode, Json)]
pub struct Culprit {
    #[json(hex)]
    pub target: OpaqueHash,
    #[json(hex)]
    pub key: Ed25519Public,
    #[json(hex)]
    pub signature: Ed25519Signature,
}

/// Represents a fault in a dispute.
#[derive(Debug, Encode, Decode, Json)]
pub struct Fault {
    #[json(hex)]
    pub target: OpaqueHash,
    pub vote: bool,
    #[json(hex)]
    pub key: Ed25519Public,
    #[json(hex)]
    pub signature: Ed25519Signature,
}

/// Represents the records of disputes.
#[derive(Debug, Encode, Decode, Json)]
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
#[derive(Debug, Encode, Decode, Json)]
pub struct DisputesExtrinsic {
    #[json(nested)]
    pub verdicts: Vec<Verdict>,
    #[json(nested)]
    pub culprits: Vec<Culprit>,
    #[json(nested)]
    pub faults: Vec<Fault>,
}
