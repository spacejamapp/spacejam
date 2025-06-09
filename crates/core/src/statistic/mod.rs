//! Statistics

use crate::{Extrinsic, TimeSlot};
use serde::{Deserialize, Serialize};
pub use {
    acc::{AccumulationRecord, AccumulationRecordJson, TransferRecord, TransferRecordJson},
    core::{CoreActivityRecord, CoreActivityRecordJson},
    service::{ServiceActivityRecord, ServiceActivityRecordJson},
    val::{ValidatorActivityRecord, ValidatorActivityRecordJson},
};

mod acc;
mod core;
mod service;
mod val;

/// Represents statistics.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Statistics {
    /// Current epoch statistics
    #[serde(rename = "vals_curr_stats")]
    pub vals_current: [ValidatorActivityRecord; crate::VALIDATORS_COUNT as usize],

    /// Last epoch statistics
    #[serde(rename = "vals_last_stats")]
    pub vals_last: [ValidatorActivityRecord; crate::VALIDATORS_COUNT as usize],

    /// Current core activity records
    #[serde(default)]
    pub cores: [CoreActivityRecord; crate::CORES_COUNT],

    /// Current service activity records
    #[serde(default)]
    pub services: Vec<(u32, ServiceActivityRecord)>,
}

impl Statistics {
    /// Update the statistics
    pub fn update(&mut self, slot: TimeSlot, index: u16, extrinsic: &Extrinsic) {
        self.update_blocks(slot, index, extrinsic);
        self.update_preimages(index, extrinsic);
        self.update_assurances(extrinsic);
        self.update_guarantees(&extrinsic);
    }

    // TODO: handle jumped blocks
    fn update_blocks(&mut self, slot: TimeSlot, index: u16, extrinsic: &Extrinsic) {
        if slot % crate::EPOCH_LENGTH == 0 {
            self.vals_last = self.vals_current;
            self.vals_current =
                [ValidatorActivityRecord::default(); crate::VALIDATORS_COUNT as usize];
        }

        self.vals_current[index as usize].blocks += 1;
        self.vals_current[index as usize].tickets += extrinsic.tickets.len() as u32;
    }

    fn update_preimages(&mut self, index: u16, extrinsic: &Extrinsic) {
        let author_stats = &mut self.vals_current[index as usize];
        author_stats.pre_images += extrinsic.preimages.len() as u32;
        author_stats.pre_images_size += extrinsic
            .preimages
            .iter()
            .map(|p| p.blob.len())
            .sum::<usize>() as u32;
    }

    fn update_assurances(&mut self, extrinsic: &Extrinsic) {
        for assurance in &extrinsic.assurances {
            self.vals_current[assurance.validator_index as usize].assurances += 1;
        }
    }

    fn update_guarantees(&mut self, extrinsic: &Extrinsic) {
        // Update Guarantees
        for guarantor in &extrinsic.guarantees {
            for signature in &guarantor.signatures {
                self.vals_current[signature.validator_index as usize].guarantees += 1;
            }
        }
    }
}
