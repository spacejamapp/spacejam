use score::{
    validator::{ValidatorDataJson, ValidatorsData},
    work::{AvailabilityAssignmentJson, AvailabilityAssignments},
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// The state of the assurance module.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct State {
    /// (ρ†) rho dagger, which is the pending reports (ϱ) after that any
    /// work report judged as uncertain or invalid has been removed from it.
    /// On success, mutated to get [ϱ‡].
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    pub avail_assignments: AvailabilityAssignments,
    /// (κ') posterior active validators.
    #[json(Vec<ValidatorDataJson>)]
    pub curr_validators: ValidatorsData,
}

impl State {
    /// Apply the state to the given state
    pub fn apply(self, state: &mut score::State) {
        state.validators.current = self.curr_validators;
        state.reports = self.avail_assignments;
    }
}

impl From<score::State> for State {
    fn from(state: score::State) -> Self {
        Self {
            avail_assignments: state.reports,
            curr_validators: state.validators.current,
        }
    }
}

impl From<&score::State> for State {
    fn from(state: &score::State) -> Self {
        Self {
            avail_assignments: state.reports.clone(),
            curr_validators: state.validators.current.clone(),
        }
    }
}

impl From<State> for score::State {
    fn from(part: State) -> Self {
        let mut state = score::State::default();
        state.validators.current = part.curr_validators;
        state.reports = part.avail_assignments;
        state
    }
}
