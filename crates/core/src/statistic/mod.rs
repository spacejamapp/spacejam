//! Statistics

use crate::{Extrinsic, TimeSlot};
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {
    acc::{AccumulationRecord, AccumulationRecordJson, TransferRecord, TransferRecordJson},
    core::{CoreActivityRecord, CoreActivityRecordJson},
    val::{ValidatorActivityRecord, ValidatorActivityRecordJson},
};

mod acc;
mod core;
mod val;

/// Represents statistics.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct Statistics {
    /// Current epoch statistics
    #[json(Vec<ValidatorActivityRecordJson>)]
    #[serde(rename = "vals_curr_stats")]
    pub vals_current: [ValidatorActivityRecord; crate::VALIDATORS_COUNT as usize],

    /// Last epoch statistics
    #[json(Vec<ValidatorActivityRecordJson>)]
    #[serde(rename = "vals_last_stats")]
    pub vals_last: [ValidatorActivityRecord; crate::VALIDATORS_COUNT as usize],

    // FIXME: workaround for async with polkavm encoding.
    #[serde(default)]
    pub workaorund: [u8; 17],
    /*  /// Current core activity records
    #[json(Vec<CoreActivityRecordJson>)]
    #[serde(default)]
    pub cores: [CoreActivityRecord; crate::CORES_COUNT as usize], */
}

impl Statistics {
    /// Update the statistics
    pub fn update(&self, slot: TimeSlot, index: u16, extrinsic: &Extrinsic) -> Self {
        let mut next = self.clone();

        // Get current and next epoch
        let new_epoch = slot % crate::EPOCH_LENGTH == 0;

        // Reset accumulator if epoch changed
        if new_epoch {
            next.vals_last = next.vals_current;
            next.vals_current =
                [ValidatorActivityRecord::default(); crate::VALIDATORS_COUNT as usize];
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
