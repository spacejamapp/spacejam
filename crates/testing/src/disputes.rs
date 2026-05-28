//! disputes test

use core::result::Result;
use runtime::tx::{self, dispute::error::Error};
use score::{
    Ed25519Public, TimeSlot,
    extrinsic::dispute::{
        DisputesExtrinsic, DisputesExtrinsicJson, DisputesRecords, DisputesRecordsJson,
    },
    safrole::{ValidatorDataJson, ValidatorsData},
    service::{AvailabilityAssignmentJson, AvailabilityAssignments},
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};

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

#[derive(Debug, Json, Serialize, Deserialize, Clone)]
pub struct Disputes {
    #[json(nested)]
    pub disputes: DisputesExtrinsic,
}

/// Test input.
#[derive(Debug, Json, Serialize, Deserialize)]
pub struct TestInput {
    #[json(nested)]
    pub input: Disputes,
    #[json(nested)]
    pub pre_state: State,
}

/// Test output.
#[derive(Debug, Json, Serialize, Deserialize)]
pub struct TestOutput {
    #[json(ResultJson<OffendersMarkJson, Error>)]
    pub output: Result<OffendersMark, Error>,
    #[json(nested)]
    pub post_state: State,
}

#[derive(Json, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct OffendersMark {
    /// [H_o] Offenders marker
    #[json(Vec<String>)]
    pub offenders_mark: Vec<Ed25519Public>,
}

#[derive(Debug, PartialEq, Eq, Json, Serialize, Deserialize, Clone)]
pub struct State {
    /// [ψ] Disputes verdicts and offenders
    #[json(nested)]
    pub psi: DisputesRecords,
    /// [ρ] Availability cores assignments
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    pub rho: AvailabilityAssignments,
    /// [τ] Timeslot
    pub tau: TimeSlot,
    /// [κ] Validators active in the current epoch
    #[json(Vec<ValidatorDataJson>)]
    pub kappa: ValidatorsData,
    /// [λ] Validators active in the previous epoch
    #[json(Vec<ValidatorDataJson>)]
    pub lambda: ValidatorsData,
}
