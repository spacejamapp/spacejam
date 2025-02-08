use crate::{
    extrinsic::AvailAssurance, BandersnatchPublic, BlsPublic, Ed25519Public, ValidatorMetadata,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Data of validators
pub type ValidatorsData = Vec<ValidatorData>;

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

impl Validators {
    /// (λ') Returns the validators for the previous epoch.
    pub fn previous(&self, new_epoch: bool) -> ValidatorsData {
        if new_epoch {
            self.current.clone()
        } else {
            self.previous.clone()
        }
    }

    /// (κ') Returns the validators for the current epoch.
    pub fn current(&self, new_epoch: bool, next: &ValidatorsData) -> ValidatorsData {
        if new_epoch {
            next.clone()
        } else {
            self.current.clone()
        }
    }
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
    /// Verify the input assurance.
    pub fn verify_assurance(&self, assurance: &AvailAssurance) -> anyhow::Result<()> {
        crypto::ed25519::verify(
            &assurance.singing_message(),
            assurance.signature,
            self.ed25519,
        )
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
