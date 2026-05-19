//! Validator data

use crate::{
    BandersnatchPublic, BlsPublic, Ed25519Public, ValidatorMetadata, block::header::EValidator,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::net::{Ipv6Addr, SocketAddrV4, SocketAddrV6};

/// Data of validators
pub type ValidatorsData = crate::Array<ValidatorData, { crate::VALIDATORS_COUNT as usize }>;

/// The validators (ι, κ, λ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default, Json)]
pub struct Validators {
    /// The validator keys and metadata to be drawn from next (ι)
    #[json(Vec<ValidatorDataJson>)]
    pub drawn: ValidatorsData,

    /// The validator keys and metadata currently active (κ)
    #[json(Vec<ValidatorDataJson>)]
    pub current: ValidatorsData,

    /// The validator keys and metadata of the previous epoch (λ)
    #[json(Vec<ValidatorDataJson>)]
    pub previous: ValidatorsData,
}

impl Validators {
    /// (λ') Returns the validators for the previous epoch.
    pub fn previous(&self, new_epoch: bool) -> &ValidatorsData {
        if new_epoch {
            &self.current
        } else {
            &self.previous
        }
    }

    /// (κ') Returns the validators for the current epoch.
    pub fn current<'a>(&'a self, new_epoch: bool, next: &'a ValidatorsData) -> &'a ValidatorsData {
        if new_epoch { next } else { &self.current }
    }
}

/// Represents the ValidatorData structure from ASN.1
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Copy)]
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
    /// Get the IPv6 address of the validator
    pub fn ipv6(&self) -> SocketAddrV6 {
        let mut addr = [0; 16];
        addr.copy_from_slice(&self.metadata[..16]);
        let port = u16::from_le_bytes([self.metadata[16], self.metadata[17]]);
        SocketAddrV6::new(Ipv6Addr::from(addr), port, 0, 0)
    }

    /// Get the IPv4 address of the validator
    pub fn ipv4(&self) -> Option<SocketAddrV4> {
        let addr = self.ipv6();
        let v4 = addr.ip().to_ipv4()?;
        Some(SocketAddrV4::new(v4, addr.port()))
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

/// Validator iterator utilities
pub trait ValidatorIter {
    /// Get the bandersnatch keys
    fn bandersnatch(&self) -> Vec<BandersnatchPublic>;

    /// Get the ed25519 keys
    fn ed25519(&self) -> Vec<Ed25519Public>;

    /// Get the epoch validators
    fn evals(&self) -> Vec<EValidator>;
}

impl<T> ValidatorIter for T
where
    T: AsRef<[ValidatorData]>,
{
    fn bandersnatch(&self) -> Vec<BandersnatchPublic> {
        self.as_ref().iter().map(|v| v.bandersnatch).collect()
    }

    fn ed25519(&self) -> Vec<Ed25519Public> {
        self.as_ref().iter().map(|v| v.ed25519).collect()
    }

    fn evals(&self) -> Vec<EValidator> {
        self.as_ref()
            .iter()
            .map(|v| EValidator {
                bandersnatch: v.bandersnatch,
                ed25519: v.ed25519,
            })
            .collect()
    }
}
