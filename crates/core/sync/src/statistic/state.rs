use score::{
    statistic::{Statistics, StatisticsJson},
    validator::{ValidatorDataJson, ValidatorsData},
    TimeSlot,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// State of the stats
///
/// NOTE: this should be moved to storage in the future
#[derive(Json, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct State {
    /// The statistics state
    #[json(nested)]
    pub pi: Statistics,
    /// The current time slot
    pub tau: TimeSlot,
    /// The current validators
    #[json(Vec<ValidatorDataJson>)]
    pub kappa_prime: ValidatorsData,
}

impl State {
    /// Apply the statistics state to the block
    pub fn apply(self, state: &mut score::State) {
        state.statistics = self.pi;
        state.validators.current = self.kappa_prime;
        state.timeslot = self.tau;
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
            pi: value.statistics,
            tau: value.timeslot,
            kappa_prime: value.validators.current,
        }
    }
}
