//! Statistics

use crate::{Extrinsic, Gas, TimeSlot};
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

/// Represents a core record.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct CoreActivityRecord {
    /// Total gas consumed by core for reported work. Includes all refinement and authorizations.
    pub gas_used: Gas,

    /// Number of segments imported from DA made by core for reported work.
    pub imports: u16,

    /// Total number of extrinsics used by core for reported work.
    pub extrinsic_count: u16,

    /// Total size of extrinsics used by core for reported work.
    pub extrinsic_size: u32,

    /// Number of segments exported into DA made by core for reported work.
    pub exports: u16,

    /// The work-bundle size. This is the size of data being placed into Audits DA by the core.
    pub bundle_size: u32,

    /// Amount of bytes which are placed into either Audits or Segments DA.
    /// This includes the work-bundle (including all extrinsics and imports) as well as all
    /// (exported) segments.
    pub da_load: u64,

    /// Number of validators which formed super-majority for assurance.
    pub popularity: u64,
}

/// Represents statistics.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct Statistics {
    /// Current epoch statistics
    #[json(nested)]
    pub vals_current: Vec<ActivityRecord>,

    /// Last epoch statistics
    #[json(nested)]
    pub vals_last: Vec<ActivityRecord>,

    /// Current core activity records
    #[json(nested)]
    pub cores: Vec<CoreActivityRecord>,
}

impl Statistics {
    /// Update the statistics
    pub fn update(&self, slot: TimeSlot, index: u16, extrinsic: &Extrinsic) -> Self {
        let mut next = self.clone();

        // Get current and next epoch
        let new_epoch = slot % crate::EPOCH_LENGTH == 0;

        // Reset accumulator if epoch changed
        if new_epoch {
            next.vals_last = next.vals_current.clone();
            next.vals_current = vec![ActivityRecord::default(); next.vals_current.len()];
        }

        // TODO: wrap this resize to the logic above.
        if next.vals_current.len() <= index as usize {
            next.vals_current
                .resize(index as usize + 1, ActivityRecord::default());
        }

        // Update block production count for author
        //
        // TODO: handle jumped blocks
        next.vals_current[index as usize].blocks += 1;
        next.vals_current[index as usize].tickets += extrinsic.tickets.len() as u32;

        // Update Preimages
        let author_stats = &mut next.vals_current[index as usize];
        author_stats.pre_images += extrinsic.preimages.len() as u32;
        author_stats.pre_images_size += extrinsic
            .preimages
            .iter()
            .map(|p| p.blob.len())
            .sum::<usize>() as u32;

        // Update Assurances
        for assurance in &extrinsic.assurances {
            next.vals_current[assurance.validator_index as usize].assurances += 1;
        }

        // Update Guarantees
        for guarantor in &extrinsic.guarantees {
            for signature in &guarantor.signatures {
                next.vals_current[signature.validator_index as usize].guarantees += 1;
            }
        }

        next
    }
}
