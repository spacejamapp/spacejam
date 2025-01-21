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
