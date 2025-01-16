//! Statistics module

use score::statistic::ActivityRecord;
pub use state::{State, StateJson};

mod state;

/// Validate the statistics state
pub fn validate(state: &mut score::State, block: &score::Block) {
    let mut pstate = State::from(state.clone());
    pstate.update(block);
    pstate.apply(state);
}

impl State {
    /// Update the statistics state
    pub fn update(&mut self, block: &score::Block) {
        let slot = block.header.slot;
        let author_index = block.header.author_index;
        let extrinsic = &block.extrinsic;

        // Get current and next epoch
        let epoch = slot / score::EPOCH_LENGTH;
        let prev_epoch = self.tau / score::EPOCH_LENGTH;

        // Reset accumulator if epoch changed
        if epoch != prev_epoch {
            self.pi.last = self.pi.current.clone();
            self.pi.current = vec![ActivityRecord::default(); self.pi.current.len()];
        }

        // Update block production count for author
        self.pi.current[author_index as usize].blocks += 1;

        // Update stats based on extrinsic type
        self.pi.current[author_index as usize].tickets += extrinsic.tickets.len() as u32;

        // Update Preimages
        let author_stats = &mut self.pi.current[author_index as usize];
        author_stats.pre_images += extrinsic.preimages.len() as u32;
        author_stats.pre_images_size += extrinsic
            .preimages
            .iter()
            .map(|p| p.blob.len())
            .sum::<usize>() as u32;

        // Update Assurances
        for assurance in &extrinsic.assurances {
            self.pi.current[assurance.validator_index as usize].assurances += 1;
        }

        // Update Guarantees
        for guarantor in &extrinsic.guarantees {
            for signature in &guarantor.signatures {
                self.pi.current[signature.validator_index as usize].guarantees += 1;
            }
        }
    }
}
