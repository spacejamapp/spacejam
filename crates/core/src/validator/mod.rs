//! Validator abstraction

use crate::{
    extrinsic::AvailAssurance, BandersnatchPublic, BlsPublic, Ed25519Public, ValidatorMetadata,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

pub use {
    context::{Context, Patch},
    extrinsic::{ExtrinsicInMem, ExtrinsicInPool},
    result::{Error, Result, ValidationError},
    validate::{
        ValidateAssurance, ValidateDispute, ValidateExtrinsic, ValidateGuarantee, ValidatePreimage,
        ValidateTicket,
    },
};

mod context;
mod extrinsic;
mod result;
mod validate;

/// The validators (ι, κ, λ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct Validators {
    /// The validator keys and metadata to be drawn from next (ι)
    pub next: ValidatorsData,

    /// The validator keys and metadata currently active (κ)
    pub current: ValidatorsData,

    /// The validator keys and metadata of the previous epoch (λ)
    pub previous: ValidatorsData,
}

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
