//! SAFRole types

use score::{
    extrinsic::ticket::{TicketBodyJson, TicketsAccumulator, TicketsOrKeys, TicketsOrKeysJson},
    validator::{ValidatorDataJson, ValidatorsData},
    BandersnatchRingCommitment, Ed25519Public, EntropyBuffer,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents the State structure.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Json, Clone)]
pub struct State {
    /// Most recent block's timeslot.
    pub tau: u32,
    /// Entropy accumulator and epochal randomness.
    ///
    /// graypaper reference: 6.21
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

impl State {
    /// Applies the state to the score state
    pub fn apply(self, state: &mut score::State) {
        state.timeslot = self.tau;
        state.entropy = self.eta;
        state.validators.previous = self.lambda;
        state.validators.current = self.kappa;
        state.validators.next = self.iota;
        state.safrole.validators = self.gamma_k;
        state.safrole.series = self.gamma_s;
        state.safrole.ring_commitment = self.gamma_z;
        state.safrole.accumulator = self.gamma_a;
        state.disputes.offenders = self.post_offenders;
    }
}

impl From<State> for score::State {
    fn from(value: State) -> Self {
        let mut state = score::State::default();
        value.apply(&mut state);
        state
    }
}

impl From<score::State> for State {
    fn from(value: score::State) -> Self {
        Self {
            tau: value.timeslot,
            eta: value.entropy,
            lambda: value.validators.previous,
            kappa: value.validators.current,
            iota: value.validators.next,
            gamma_k: value.safrole.validators,
            gamma_z: value.safrole.ring_commitment,
            gamma_s: value.safrole.series,
            gamma_a: value.safrole.accumulator,
            post_offenders: value.disputes.offenders,
        }
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
