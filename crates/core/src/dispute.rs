//! Dispute types

use crate::misc::*;

/// Represents a judgement in a dispute.
pub struct Judgement {
    pub vote: bool,                  // Corresponds to BOOLEAN
    pub index: ValidatorIndex,       // Corresponds to ValidatorIndex
    pub signature: Ed25519Signature, // Corresponds to Ed25519Signature
}

/// Represents a verdict in a dispute.
pub struct Verdict {
    pub target: OpaqueHash,    // Corresponds to OpaqueHash
    pub age: u32,              // Corresponds to U32
    pub votes: Vec<Judgement>, // Sequence of Judgement
}

/// Represents a culprit in a dispute.
pub struct Culprit {
    pub target: OpaqueHash,          // Corresponds to WorkReportHash
    pub key: Ed25519Public,          // Corresponds to Ed25519Public
    pub signature: Ed25519Signature, // Corresponds to Ed25519Signature
}

/// Represents a fault in a dispute.
pub struct Fault {
    pub target: OpaqueHash,          // Corresponds to WorkReportHash
    pub vote: bool,                  // Corresponds to BOOLEAN
    pub key: Ed25519Public,          // Corresponds to Ed25519Public
    pub signature: Ed25519Signature, // Corresponds to Ed25519Signature
}

/// Represents the records of disputes.
pub struct DisputesRecords {
    pub good: Vec<OpaqueHash>,         // Sequence of WorkReportHash
    pub bad: Vec<OpaqueHash>,          // Sequence of WorkReportHash
    pub wonky: Vec<OpaqueHash>,        // Sequence of WorkReportHash
    pub offenders: Vec<Ed25519Public>, // Sequence of Ed25519Public
}

/// Represents the extrinsic data for disputes.
pub struct DisputesExtrinsic {
    pub verdicts: Vec<Verdict>, // Sequence of Verdict
    pub culprits: Vec<Culprit>, // Sequence of Culprit
    pub faults: Vec<Fault>,     // Sequence of Fault
}
