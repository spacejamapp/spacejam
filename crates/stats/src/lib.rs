use score::{
    block::Extrinsic,
    misc::{
        ActivityRecord, Ed25519Public, Statistics, StatisticsJson, TimeSlot, ValidatorDataJson,
        ValidatorIndex, ValidatorsData,
    },
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// State of the stats
///
/// NOTE: this should be moved to storage in the future
#[derive(Json, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct State {
    #[json(nested)]
    pub pi: Statistics,
    pub tau: TimeSlot,
    #[json(Vec<ValidatorDataJson>)]
    pub kappa_prime: ValidatorsData,
}

/// Registry of the stats
pub struct Stats {
    pub state: State,
    pub next_state: State,
}

impl From<State> for Stats {
    fn from(state: State) -> Self {
        Self {
            next_state: state.clone(),
            state,
        }
    }
}

impl Stats {
    pub fn update(
        mut self,
        slot: TimeSlot,
        author_index: ValidatorIndex,
        extrinsic: Extrinsic,
        reporters: Vec<Ed25519Public>,
    ) -> Self {
        // Get current and next epoch
        let epoch = slot / score::EPOCH_LENGTH;
        let prev_epoch = self.state.tau / score::EPOCH_LENGTH;

        // Reset accumulator if epoch changed
        if epoch != prev_epoch {
            self.next_state.pi.current =
                vec![ActivityRecord::default(); self.state.pi.current.len()];
            self.next_state.pi.last = self.state.pi.current.clone();
        }

        // Update block production count for author
        self.next_state.pi.current[author_index as usize].blocks += 1;

        // Update stats based on extrinsic type
        self.next_state.pi.current[author_index as usize].tickets += extrinsic.tickets.len() as u32;

        // Update Preimages
        let author_stats = &mut self.next_state.pi.current[author_index as usize];
        author_stats.pre_images += extrinsic.preimages.len() as u32;
        author_stats.pre_images_size += extrinsic
            .preimages
            .iter()
            .map(|p| p.blob.len())
            .sum::<usize>() as u32;

        // Update Assurances
        for assurance in extrinsic.assurances {
            self.next_state.pi.current[assurance.validator_index as usize].assurances += 1;
        }

        // Update guarantor reports
        for reporter in reporters {
            if let Some(validator_idx) = self
                .state
                .kappa_prime
                .iter()
                .position(|v| v.ed25519 == reporter)
            {
                self.next_state.pi.current[validator_idx].guarantees += 1;
            }
        }

        // Update timestamp
        self.next_state.tau = slot;

        self
    }
}
