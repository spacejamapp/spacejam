//! Dispute types

use crate::{misc::*, JAM_GUARANTEE, JAM_INVALID, JAM_VALID, VALIDATORS_SUPER_MAJORITY};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a judgement in a dispute.
#[derive(Debug, Serialize, Deserialize, Json, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Judgement {
    pub vote: bool,
    pub index: ValidatorIndex,
    #[json(hex)]
    #[serde(with = "codec::bytes")]
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
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Verdict {
    #[json(hex)]
    pub target: OpaqueHash,
    /// Age of the verdict
    pub age: u32,
    #[json(Vec<JudgementJson>)]
    pub votes: [Judgement; VALIDATORS_SUPER_MAJORITY as usize],
}

impl Verdict {
    /// Returns the message that was signed by the verdict.
    pub fn signature_message(&self, vote: bool) -> Vec<u8> {
        let mut message = vec![];
        if vote {
            message.extend_from_slice(&JAM_VALID);
        } else {
            message.extend_from_slice(&JAM_INVALID);
        }
        message.extend_from_slice(&self.target);
        message
    }
}

/// Represents a culprit in a dispute.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Culprit {
    #[json(hex)]
    pub target: OpaqueHash,
    #[json(hex)]
    pub key: Ed25519Public,
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub signature: Ed25519Signature,
}

impl Culprit {
    /// Returns the message that was signed by the culprit.
    pub fn signature_message(&self) -> [u8; 45] {
        let mut message = [0; 45];
        message[0..13].copy_from_slice(&JAM_GUARANTEE);
        message[13..45].copy_from_slice(&self.target);
        message
    }

    /// Verifies the signature of the culprit.
    pub fn verify(&self) -> anyhow::Result<()> {
        crypto::ed25519::verify(&self.signature_message(), self.signature, self.key)
    }
}

/// Represents a fault in a dispute.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Fault {
    #[json(hex)]
    pub target: OpaqueHash,
    pub vote: bool,
    #[json(hex)]
    pub key: Ed25519Public,
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub signature: Ed25519Signature,
}

impl Fault {
    /// Returns the message that was signed by the fault.
    pub fn singing_message(&self) -> Vec<u8> {
        let mut message = vec![];
        if self.vote {
            message.extend_from_slice(&JAM_VALID);
        } else {
            message.extend_from_slice(&JAM_INVALID);
        }
        message.extend_from_slice(&self.target);
        message
    }

    /// Verifies the signature of the fault.
    pub fn verify(&self) -> anyhow::Result<()> {
        crypto::ed25519::verify(&self.singing_message(), self.signature, self.key)
    }
}

/// Represents the records of disputes.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
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
