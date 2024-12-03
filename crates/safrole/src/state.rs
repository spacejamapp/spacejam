//! SAFRole types

use crate::Error;
use anyhow::Result;
use codec::Json;
use core::{
    block::header::{EpochMark, EpochMarkJson, TicketsMark},
    misc::{
        BandersnatchRingCommitment, EntropyBuffer, OpaqueHash, ValidatorDataJson, ValidatorsData,
    },
    ticket::{
        TicketBodyJson, TicketsAccumulator, TicketsExtrinsic, TicketsOrKeys, TicketsOrKeysJson,
    },
};
use serde::{Deserialize, Serialize};

/// Represents the State structure.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Json)]
pub struct State {
    /// Current epoch
    pub tau: u32,
    /// Entropy accumulator
    #[json(Vec<String>)]
    pub eta: EntropyBuffer,
    /// Previous epoch's validators
    #[json(Vec<ValidatorDataJson>)]
    pub lambda: ValidatorsData,
    /// Current epoch's validators
    #[json(Vec<ValidatorDataJson>)]
    pub kappa: ValidatorsData,
    /// Validators to be drawn from next
    #[json(Vec<ValidatorDataJson>)]
    pub iota: ValidatorsData,
    /// Next epoch's validators
    #[json(Vec<ValidatorDataJson>)]
    pub gamma_k: ValidatorsData,
    /// Bandersnatch ring commitment
    #[serde(with = "codec")]
    #[json(hex)]
    pub gamma_z: BandersnatchRingCommitment,
    /// Sealing-key series of the current epoch
    #[json(nested)]
    pub gamma_s: TicketsOrKeys,
    /// Sealing-key contest ticket accumulator
    #[json(Vec<TicketBodyJson>)]
    pub gamma_a: TicketsAccumulator,
}

impl State {
    /// Enacts an epoch change.
    pub fn enact(
        &mut self,
        slot: u32,
        _entropy: OpaqueHash,
        _extrinsic: TicketsExtrinsic,
    ) -> Result<std::result::Result<OutputData, Error>> {
        if slot <= self.tau {
            return Ok(Err(Error::BadSlot));
        }

        self.tau = slot;

        Ok(Ok(OutputData::default()))
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
        }
    }
}

/// Represents the Output marks
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Json)]
pub struct OutputData {
    /// New epoch marker
    #[json(nested)]
    pub epoch_mark: Option<EpochMark>,
    /// New tickets marker
    #[json(Option<Vec<TicketBodyJson>>)]
    pub tickets_mark: Option<TicketsMark>,
}
