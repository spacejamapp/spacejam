//! disputes test

use core::result::Result;
use runtime::tx::{self, dispute::error::Error};
use score::{
    Ed25519Public, TimeSlot,
    extrinsic::dispute::{DisputesExtrinsic, DisputesRecords},
    safrole::ValidatorsData,
    service::AvailabilityAssignments,
};
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/disputes.rs"));

pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let (input, pre_state, out, post_state) =
        codec::decode::<(Disputes, State, Result<OffendersMark, Error>, State)>(
            test.input.expect_bin()?,
        )?;
    let mut input = TestInput { input, pre_state };
    let output = TestOutput {
        output: out,
        post_state,
    };
    let result = tx::dispute::disputes(
        input.pre_state.tau,
        &input.pre_state.kappa,
        &input.pre_state.lambda,
        input.pre_state.psi.clone(),
        &input.input.disputes,
    )
    .and_then(|(next_psi, records, triples)| {
        crypto::ed25519::SigItem::batch_verify(&triples).map_err(|_| Error::BadSignature)?;
        Ok((next_psi, records))
    });

    // check offenders mark
    assert_eq!(
        result.clone().map(|(_, mark)| mark.offenders),
        output.output.map(|v| { v.offenders_mark })
    );

    if let Ok((psi, records)) = result {
        input.pre_state.psi = psi;
        input.pre_state.rho = tx::dispute::reports(&records, &input.pre_state.rho);
    }

    // check post state
    assert_eq!(input.pre_state, output.post_state);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Disputes {
    pub disputes: DisputesExtrinsic,
}

/// Test input.
#[derive(Debug, Serialize, Deserialize)]
pub struct TestInput {
    pub input: Disputes,
    pub pre_state: State,
}

/// Test output.
#[derive(Debug, Serialize, Deserialize)]
pub struct TestOutput {
    pub output: Result<OffendersMark, Error>,
    pub post_state: State,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct OffendersMark {
    /// [H_o] Offenders marker
    pub offenders_mark: Vec<Ed25519Public>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct State {
    /// [ψ] Disputes verdicts and offenders
    pub psi: DisputesRecords,
    /// [ρ] Availability cores assignments
    pub rho: AvailabilityAssignments,
    /// [τ] Timeslot
    pub tau: TimeSlot,
    /// [κ] Validators active in the current epoch
    pub kappa: ValidatorsData,
    /// [λ] Validators active in the previous epoch
    pub lambda: ValidatorsData,
}
