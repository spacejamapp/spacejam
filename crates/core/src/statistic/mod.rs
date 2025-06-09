//! Statistics

use std::collections::BTreeMap;

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
    pub services: BTreeMap<u32, ServiceActivityRecord>,
}

impl Statistics {
    /// Update the statistics
    pub fn update(&mut self, slot: TimeSlot, index: u16, extrinsic: &Extrinsic) {
        self.update_blocks(slot, index, extrinsic);
        self.update_preimages(index, extrinsic);
        self.update_assurances(extrinsic);
        self.update_guarantees(&extrinsic);
    }

    /// Merge the service statistics
    pub fn merge_services(&mut self, services: BTreeMap<u32, ServiceActivityRecord>) {
        for (service, record) in services {
            if let Some(entry) = self.services.get_mut(&service) {
                entry.accumulate_count = record.accumulate_count;
                entry.accumulate_gas_used = record.accumulate_gas_used;
            } else {
                self.services.insert(service, record);
            }
        }
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

    // TODO:
    // - [ ] core: da_load and popularity
    // - [ ] service: missing parts
    fn update_guarantees(&mut self, extrinsic: &Extrinsic) {
        for report in &extrinsic.guarantees {
            for signature in &report.signatures {
                self.vals_current[signature.validator_index as usize].guarantees += 1;
            }

            // Update core and service statistics
            let core = &mut self.cores[report.report.core_index as usize];
            core.bundle_size += report.report.spec.length;
            for result in &report.report.results {
                core.imports += result.refine_load.imports;
                core.exports += result.refine_load.exports;
                core.extrinsic_count += result.refine_load.extrinsic_count;
                core.extrinsic_size += result.refine_load.extrinsic_size;
                core.gas_used += result.refine_load.gas_used;

                let service = self
                    .services
                    .entry(result.service_id.into())
                    .or_insert_with(|| ServiceActivityRecord::default());
                service.extrinsic_count += result.refine_load.extrinsic_count as u32;
                service.extrinsic_size += result.refine_load.extrinsic_size as u32;
                service.refinement_gas_used += result.refine_load.gas_used;
                service.refinement_count += 1;
            }
        }
    }
}
