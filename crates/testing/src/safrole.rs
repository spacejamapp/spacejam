//! Safrole vector tests

use runtime::tx::{self, ticket::Error};
use score::{
    block::header::{EpochMark, EpochMarkJson, TicketsMark},
    extrinsic::ticket::{
        TicketBodyJson, TicketEnvelopeJson, TicketsAccumulator, TicketsExtrinsic, TicketsOrKeys,
        TicketsOrKeysJson,
    },
    safrole::{Safrole, ValidatorDataJson, Validators, ValidatorsData},
    BandersnatchRingCommitment, Ed25519Public, EntropyBuffer, OpaqueHash,
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};

/// Test input.
#[derive(Deserialize, Serialize, Json, Debug)]
pub struct Input {
    pub slot: u32,
    #[json(hex)]
    pub entropy: OpaqueHash,
    #[json(Vec<TicketEnvelopeJson>)]
    pub extrinsic: TicketsExtrinsic,
}

/// Test input.
#[derive(Deserialize, Serialize, Json, Debug)]
pub struct TestInput {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: State,
}

/// Test output.
#[derive(Deserialize, Serialize, Json, Debug)]
pub struct TestOutput {
    #[json(ResultJson<MarkersJson, Error>)]
    pub output: std::result::Result<Markers, Error>,
    #[json(nested)]
    pub post_state: State,
}

crate::impl_tests! {
    safrole,
    @scale
    enact_epoch_change_with_no_tickets_1,
    enact_epoch_change_with_no_tickets_2,
    enact_epoch_change_with_no_tickets_3,
    enact_epoch_change_with_no_tickets_4,
    enact_epoch_change_with_padding_1,
    publish_tickets_no_mark_1,
    publish_tickets_no_mark_2,
    publish_tickets_no_mark_3,
    publish_tickets_no_mark_4,
    publish_tickets_no_mark_5,
    publish_tickets_no_mark_6,
    publish_tickets_no_mark_7,
    publish_tickets_no_mark_8,
    publish_tickets_no_mark_9,
    publish_tickets_with_mark_1,
    publish_tickets_with_mark_2,
    publish_tickets_with_mark_3,
    publish_tickets_with_mark_4,
    publish_tickets_with_mark_5,
    skip_epoch_tail_1,
    skip_epochs_1
}

/// Represents the Output marks
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Json)]
pub struct Markers {
    /// New epoch marker
    #[json(nested)]
    pub epoch_mark: Option<EpochMark>,
    /// New tickets marker
    #[json(Option<Vec<TicketBodyJson>>)]
    pub tickets_mark: Option<TicketsMark>,
}

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
    /// Enacts the epoch change and updates the state.
    pub fn enact(&mut self, input: &Input) -> Result<Markers, Error> {
        let prev = self.clone();
        let new_epoch = input.slot / score::EPOCH_LENGTH > self.tau / score::EPOCH_LENGTH;
        let safrole = Safrole {
            validators: self.gamma_k.clone(),
            series: self.gamma_s.clone(),
            ring_commitment: self.gamma_z.clone(),
            accumulator: self.gamma_a.clone(),
        };

        let mut validators = Validators {
            current: self.kappa.clone(),
            next: self.iota.clone(),
            previous: self.lambda.clone(),
        };

        validators = tx::ticket::validators(new_epoch, &safrole.validators, &validators);
        self.eta = tx::ticket::eta(new_epoch, &self.eta, input.entropy);

        let mut markers = Markers::default();
        match tx::ticket::safrole(
            self.tau,
            input.slot,
            self.eta,
            &self.post_offenders,
            &safrole,
            &validators,
            &input.extrinsic,
        ) {
            Ok(safrole) => {
                markers.epoch_mark = safrole.epoch_mark(new_epoch, &self.eta);
                markers.tickets_mark = safrole.tickets_mark(self.tau, input.slot);

                self.gamma_a = safrole.accumulator;
                self.gamma_k = safrole.validators;
                self.gamma_s = safrole.series;
                self.gamma_z = safrole.ring_commitment;
                self.kappa = validators.current;
                self.lambda = validators.previous;
                self.tau = input.slot;
            }
            Err(e) => {
                *self = prev;
                return Err(e);
            }
        }

        Ok(markers)
    }
}
