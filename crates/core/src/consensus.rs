use crate::{
    extrinsic::ticket::{TicketBodyJson, TicketsAccumulator, TicketsOrKeys, TicketsOrKeysJson},
    misc::{
        BandersnatchRingCommitment, Ed25519Public, EntropyBuffer, ValidatorDataJson, ValidatorsData,
    },
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Safrole consensus system state
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Json, Clone)]
pub struct Safrole {
    /// Most recent block's timeslot.
    pub tau: u32,
    /// Entropy accumulator and epochal randomness.
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
    #[serde(with = "codec::bytes")]
    #[json(hex)]
    pub gamma_z: BandersnatchRingCommitment,
    /// Sealing-key series of the current epoch
    #[json(nested)]
    pub gamma_s: TicketsOrKeys,
    /// Sealing-key contest ticket accumulator
    #[json(Vec<TicketBodyJson>)]
    pub gamma_a: TicketsAccumulator,
    /// Offenders
    #[json(Vec<String>)]
    pub post_offenders: Vec<Ed25519Public>,
}
