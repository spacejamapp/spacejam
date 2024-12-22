use crate::{
    extrinsic::{TicketsAccumulator, TicketsOrKeys},
    validator::ValidatorsData,
    BandersnatchRingCommitment,
};
use serde::{Deserialize, Serialize};

/// Safrole consensus state
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Safrole {
    /// Sealing-key contest ticket accumulator (gamma_a)
    pub accumulator: TicketsAccumulator,
    /// Next epoch's validators (gamma_k)
    pub validators: ValidatorsData,
    /// Sealing-key series of the current epoch (gamma_s)
    pub series: TicketsOrKeys,
    /// Bandersnatch ring commitment (gamma_z)
    #[serde(with = "codec::bytes")]
    pub ring_commitment: BandersnatchRingCommitment,
}

impl Default for Safrole {
    fn default() -> Self {
        Self {
            accumulator: vec![],
            validators: vec![],
            series: TicketsOrKeys::default(),
            ring_commitment: [0u8; 144],
        }
    }
}
