//! Disputes state

use score::{
    extrinsic::dispute::{DisputesRecords, DisputesRecordsJson},
    validator::{ValidatorDataJson, ValidatorsData},
    work::{AvailabilityAssignment, AvailabilityAssignmentJson},
    TimeSlot,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

#[derive(Debug, PartialEq, Eq, Json, Serialize, Deserialize, Clone)]
pub struct State {
    /// [ψ] Disputes verdicts and offenders
    #[json(nested)]
    pub psi: DisputesRecords,
    /// [ρ] Availability cores assignments
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    pub rho: Vec<Option<AvailabilityAssignment>>,
    /// [τ] Timeslot
    pub tau: TimeSlot,
    /// [κ] Validators active in the current epoch
    #[json(Vec<ValidatorDataJson>)]
    pub kappa: ValidatorsData,
    /// [λ] Validators active in the previous epoch
    #[json(Vec<ValidatorDataJson>)]
    pub lambda: ValidatorsData,
}

impl From<score::State> for State {
    fn from(state: score::State) -> Self {
        State {
            psi: state.disputes,
            rho: state.reports.to_vec(),
            tau: state.timeslot,
            kappa: state.validators.current.to_vec(),
            lambda: state.validators.previous.to_vec(),
        }
    }
}

impl From<State> for score::State {
    fn from(state: State) -> Self {
        let mut target = score::State::default();
        target.disputes = state.psi;
        for (i, assignment) in state.rho.iter().enumerate() {
            target.reports[i] = assignment.clone();
        }
        target.timeslot = state.tau;
        target.validators.current = state.kappa.into_iter().collect();
        target.validators.previous = state.lambda.into_iter().collect();
        target
    }
}
