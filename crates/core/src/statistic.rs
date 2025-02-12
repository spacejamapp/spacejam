//! Statistics

use crate::{Extrinsic, TimeSlot};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents an activity record.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct ActivityRecord {
    /// Number of blocks produced
    pub blocks: u32,
    /// Number of tickets
    pub tickets: u32,
    /// Number of pre-images
    pub pre_images: u32,
    /// Size of pre-images
    pub pre_images_size: u32,
    /// Number of guarantees
    pub guarantees: u32,
    /// Number of assurances
    pub assurances: u32,
}

/// Represents statistics.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct Statistics {
    /// Current epoch statistics
    #[json(nested)]
    pub current: Vec<ActivityRecord>,
    /// Last epoch statistics
    #[json(nested)]
    pub last: Vec<ActivityRecord>,
}

impl Statistics {
    /// Update the statistics
    pub fn update(&self, tau: TimeSlot, slot: TimeSlot, index: u16, extrinsic: &Extrinsic) -> Self {
        let mut next = self.clone();

        // Get current and next epoch
        let epoch = slot / crate::EPOCH_LENGTH;
        let prev_epoch = tau / crate::EPOCH_LENGTH;

        // Reset accumulator if epoch changed
        if epoch != prev_epoch {
            next.last = next.current.clone();
            next.current = vec![ActivityRecord::default(); next.current.len()];
        }

        // Update block production count for author
        //
        // TODO: handle jumped blocks
        next.current[index as usize].blocks += 1;
        next.current[index as usize].tickets += extrinsic.tickets.len() as u32;

        // Update Preimages
        let author_stats = &mut next.current[index as usize];
        author_stats.pre_images += extrinsic.preimages.len() as u32;
        author_stats.pre_images_size += extrinsic
            .preimages
            .iter()
            .map(|p| p.blob.len())
            .sum::<usize>() as u32;

        // Update Assurances
        for assurance in &extrinsic.assurances {
            next.current[assurance.validator_index as usize].assurances += 1;
        }

        // Update Guarantees
        for guarantor in &extrinsic.guarantees {
            for signature in &guarantor.signatures {
                next.current[signature.validator_index as usize].guarantees += 1;
            }
        }

        next
    }
}
