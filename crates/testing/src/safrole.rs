//! Safrole vector tests

use runtime::tx::{self, ticket::Error};
use score::{
    BandersnatchRingCommitment, Ed25519Public, EntropyBuffer, OpaqueHash,
    block::header::{EpochMark, TicketsMark},
    extrinsic::ticket::{TicketsAccumulator, TicketsExtrinsic, TicketsOrKeys},
    safrole::{Safrole, Validators, ValidatorsData},
};
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/safrole.rs"));

/// Run the safrole test
pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let (input, pre_state, out, post_state) =
        codec::decode::<(Input, State, std::result::Result<Markers, Error>, State)>(
            test.input.expect_bin()?,
        )?;
    let mut input = TestInput { input, pre_state };
    let output = TestOutput {
        output: out,
        post_state,
    };
    let result = input.pre_state.enact(&input.input);

    assert_eq!(result, output.output);
    assert_eq!(output.post_state.gamma_a, input.pre_state.gamma_a);
    assert_eq!(output.post_state.gamma_k, input.pre_state.gamma_k);
    assert_eq!(output.post_state.gamma_s, input.pre_state.gamma_s);
    assert_eq!(output.post_state.gamma_z, input.pre_state.gamma_z);
    assert_eq!(output.post_state, input.pre_state);
    Ok(())
}

/// Test input.
#[derive(Deserialize, Serialize, Debug)]
pub struct Input {
    pub slot: u32,
    pub entropy: OpaqueHash,
    pub extrinsic: TicketsExtrinsic,
}

/// Test input.
#[derive(Deserialize, Serialize, Debug)]
pub struct TestInput {
    pub input: Input,
    pub pre_state: State,
}

/// Test output.
#[derive(Deserialize, Serialize, Debug)]
pub struct TestOutput {
    pub output: std::result::Result<Markers, Error>,
    pub post_state: State,
}

/// Represents the Output marks
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Markers {
    /// New epoch marker
    pub epoch_mark: Option<EpochMark>,
    /// New tickets marker
    pub tickets_mark: Option<TicketsMark>,
}

/// Represents the State structure.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct State {
    /// Most recent block's timeslot.
    pub tau: u32,
    /// Entropy accumulator and epochal randomness.
    ///
    /// graypaper reference: 6.21
    pub eta: EntropyBuffer,
    /// Previous epoch's validators
    pub lambda: ValidatorsData,
    /// Current epoch's validators
    pub kappa: ValidatorsData,
    /// Next epoch's validators
    pub gamma_k: ValidatorsData,
    /// Validators to be drawn from next
    pub iota: ValidatorsData,
    /// Sealing-key contest ticket accumulator
    pub gamma_a: TicketsAccumulator,
    /// Sealing-key series of the current epoch
    pub gamma_s: TicketsOrKeys,
    /// Bandersnatch ring commitment
    #[serde(with = "codec::bytes")]
    pub gamma_z: BandersnatchRingCommitment,
    /// Offenders
    pub post_offenders: Vec<Ed25519Public>,
}

impl State {
    /// Enacts the epoch change and updates the state.
    pub fn enact(&mut self, input: &Input) -> Result<Markers, Error> {
        let prev = self.clone();
        let new_epoch = input.slot / score::EPOCH_LENGTH > self.tau / score::EPOCH_LENGTH;
        let safrole = Safrole {
            validators: std::mem::take(&mut self.gamma_k),
            series: std::mem::take(&mut self.gamma_s),
            ring_commitment: self.gamma_z,
            accumulator: std::mem::take(&mut self.gamma_a),
        };

        let mut validators = Validators {
            current: std::mem::take(&mut self.kappa),
            drawn: std::mem::take(&mut self.iota),
            previous: std::mem::take(&mut self.lambda),
        };

        validators = tx::ticket::validators(new_epoch, &safrole.validators, &validators);
        self.eta = tx::ticket::eta(new_epoch, &self.eta, input.entropy);

        let mut markers = Markers::default();
        match tx::ticket::safrole(
            self.tau,
            input.slot,
            self.eta,
            &self.post_offenders,
            safrole.clone(),
            &validators,
            &input.extrinsic,
        ) {
            Ok(safrole) => {
                if new_epoch {
                    markers.epoch_mark = safrole.epoch_mark(&self.eta);
                }
                markers.tickets_mark = safrole.tickets_mark(self.tau, input.slot);

                self.gamma_a = safrole.accumulator;
                self.gamma_k = safrole.validators;
                self.gamma_s = safrole.series;
                self.gamma_z = safrole.ring_commitment;
                self.kappa = validators.current;
                self.lambda = validators.previous;
                self.iota = validators.drawn;
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
