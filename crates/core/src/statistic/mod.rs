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
        for core in &mut self.cores {
            *core = CoreActivityRecord::default();
        }

        self.services.clear();
        self.update_blocks(slot, index, extrinsic);
        self.update_preimages(index, extrinsic);
        self.update_assurances(extrinsic);
        self.update_guarantees(&extrinsic);
    }

    /// Merge the service statistics from accumulation
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

    /// Update transfer statistics for services
    pub fn merge_transfers(&mut self, transfers: BTreeMap<u32, (usize, crate::Gas)>) {
        for (service_id, (count, gas)) in transfers {
            let service = self
                .services
                .entry(service_id)
                .or_insert_with(|| ServiceActivityRecord::default());
            service.on_transfers_count += count as u32;
            service.on_transfers_gas_used += gas;
        }
    }

    /// Update popularity statistics based on assurances
    ///
    /// This should be called with assurances that led to newly available work reports
    /// Gray Paper: p ≡ ∑_{a ∈ E_A} a_f[c] where a_f is the assurance count per core
    pub fn merge_reports(
        &mut self,
        available: &[crate::service::WorkReport],
        assurances: &[u32; crate::CORES_COUNT],
    ) {
        // Update DA load and popularity only for newly available reports
        if !available.is_empty() {
            self.update_available_reports(&available);
            self.update_popularity(&available, assurances);
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
        // Update validator statistics (Gray Paper equation for π'_V[v]_p and π'_V[v]_d)
        let author_stats = &mut self.vals_current[index as usize];
        author_stats.pre_images += extrinsic.preimages.len() as u32;
        author_stats.pre_images_size += extrinsic
            .preimages
            .iter()
            .map(|p| p.blob.len())
            .sum::<usize>() as u32;

        // Update service statistics for provided preimages (Gray Paper πS[s]_p)
        // According to GP: p ≡ ∑_{(s,𝐩) ∈ 𝐄_P} (1, |𝐩|)
        for preimage in &extrinsic.preimages {
            let service = self
                .services
                .entry(preimage.requester)
                .or_insert_with(|| ServiceActivityRecord::default());
            service.provided_count += 1;
            service.provided_size += preimage.blob.len() as u32;
        }
    }

    fn update_assurances(&mut self, extrinsic: &Extrinsic) {
        for assurance in &extrinsic.assurances {
            self.vals_current[assurance.validator_index as usize].assurances += 1;
        }
    }

    fn update_guarantees(&mut self, extrinsic: &Extrinsic) {
        for report in &extrinsic.guarantees {
            // Update validator statistics for guarantees (Gray Paper π'_V[v]_g)
            for signature in &report.signatures {
                self.vals_current[signature.validator_index as usize].guarantees += 1;
            }

            let core = &mut self.cores[report.report.core_index as usize];
            core.bundle_size += report.report.spec.length;
            for result in &report.report.results {
                core.imports += result.refine_load.imports;
                core.exports += result.refine_load.exports;
                core.extrinsic_count += result.refine_load.extrinsic_count;
                core.extrinsic_size += result.refine_load.extrinsic_size;
                core.gas_used += result.refine_load.gas_used;

                // Update service statistics
                let service = self
                    .services
                    .entry(result.service_id.into())
                    .or_insert_with(|| ServiceActivityRecord::default());

                service.refinement_count += 1;
                service.refinement_gas_used += result.refine_load.gas_used;
                service.imports += result.refine_load.imports as u32;
                service.exports += result.refine_load.exports as u32;
                service.extrinsic_count += result.refine_load.extrinsic_count as u32;
                service.extrinsic_size += result.refine_load.extrinsic_size as u32;
            }
        }
    }

    /// Update DA load and popularity statistics from newly available work reports
    /// This should be called with 𝐖 (newly available work-reports) from assurance processing
    /// Gray Paper: D(c) ≡ ∑_{w ∈ 𝐖, w_c = c} (w_s)_l + 𝐖_G⌈(w_s)_n·65/64⌉
    fn update_available_reports(&mut self, available_reports: &[crate::service::WorkReport]) {
        // Calculate DA load for each core based on newly available work reports
        for report in available_reports {
            let core = &mut self.cores[report.core_index as usize];

            // Gray Paper: D(c) ≡ ∑_{w ∈ 𝐖, w_c = c} (w_s)_l + 𝐖_G⌈(w_s)_n·65/64⌉
            // where (w_s)_l is the work package length and (w_s)_n is the exports_count (number of segments)
            let segment_overhead = (report.spec.exports_count as u64)
                .saturating_mul(65)
                .div_ceil(64)
                .saturating_mul(crate::SEGMENT_SIZE as u64); // 𝐖_G = segment size
            let da_load_delta = report.spec.length + segment_overhead as u32;
            core.da_load += da_load_delta;
        }
    }

    /// Update popularity statistics based on assurance super-majority
    /// This should be called after processing assurances for available work reports
    fn update_popularity(
        &mut self,
        available: &[crate::service::WorkReport],
        assurance_counts: &[u32; crate::CORES_COUNT],
    ) {
        // (p) Popularity - Number of validators which formed super-majority for assurance
        // Gray Paper: p ≡ ∑_{a ∈ E_A} a_f[c] where a_f is the assurance count per core
        for (core_index, &assurance_count) in assurance_counts.iter().enumerate() {
            if assurance_count >= crate::VALIDATORS_SUPER_MAJORITY as u32 {
                // Only count if the core had an available report that reached super-majority
                if available
                    .iter()
                    .any(|report| report.core_index as usize == core_index)
                {
                    self.cores[core_index].popularity += assurance_count as u16;
                }
            }
        }
    }
}
