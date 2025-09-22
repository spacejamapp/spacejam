//! Guarantee types

use crate::{
    Ed25519Signature, TimeSlot, ValidatorIndex,
    service::{WorkReport, WorkReportJson},
};
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
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct ReportGuarantee {
    /// The report
    #[json(nested)]
    pub report: WorkReport,

    /// The slot
    pub slot: TimeSlot,

    /// The signatures
    #[json(nested)]
    pub signatures: Vec<ValidatorSignature>,
}

impl ReportGuarantee {
    #[cfg(feature = "blake2")]
    /// Returns the message that was signed by the guarantors.
    pub fn signing_message(&self) -> anyhow::Result<Vec<u8>> {
        let mut message = vec![];
        message.extend_from_slice(&crate::JAM_GUARANTEE);

        let hashed = crate::blake2b(&codec::encode(&self.report)?);
        message.extend_from_slice(&hashed);
        Ok(message)
    }
}

/// Represents a sequence of guarantees.
pub type GuaranteesExtrinsic = Vec<ReportGuarantee>;
