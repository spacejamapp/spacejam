//! SAFRole types

use crate::Error;
use anyhow::Result;
use core::{
    block::header::{EpochMark, TicketsMark},
    misc::{BandersnatchRingCommitment, Ed25519Public, EntropyBuffer, OpaqueHash, ValidatorsData},
    ticket::{TicketsAccumulator, TicketsExtrinsic, TicketsOrKeys},
};
use serde::{Deserialize, Serialize};

/// Represents the State structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    /// Current epoch
    pub tau: u32,
    /// Entropy accumulator
    pub eta: EntropyBuffer,
    /// Previous epoch's validators
    pub lambda: ValidatorsData,
    /// Current epoch's validators
    pub kappa: ValidatorsData,
    /// Validators to be drawn from next
    pub iota: ValidatorsData,
    /// Next epoch's validators
    pub gamma_k: ValidatorsData,
    /// Bandersnatch ring commitment
    #[serde(with = "codec")]
    pub gamma_z: BandersnatchRingCommitment,
    /// Sealing-key series of the current epoch
    pub gamma_s: TicketsOrKeys,
    /// Sealing-key contest ticket accumulator
    pub gamma_a: TicketsAccumulator,
    /// Posterior offenders sequence
    pub post_offenders: Vec<Ed25519Public>,
}

impl State {
    /// Enacts an epoch change.
    pub fn enact(
        &mut self,
        _slot: u32,
        _entropy: OpaqueHash,
        _extrinsic: TicketsExtrinsic,
    ) -> Result<std::result::Result<OutputData, Error>> {
        todo!()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            tau: 0,
            eta: Default::default(),
            lambda: Default::default(),
            kappa: Default::default(),
            iota: Default::default(),
            gamma_k: Default::default(),
            gamma_z: [0u8; 144],
            gamma_s: Default::default(),
            gamma_a: Default::default(),
            post_offenders: Default::default(),
        }
    }
}

/// Represents the Output marks
#[derive(Serialize, Deserialize, Debug)]
pub struct OutputData {
    /// New epoch marker
    pub epoch_mark: Option<EpochMark>,
    /// New tickets marker
    pub tickets_mark: Option<TicketsMark>,
}
