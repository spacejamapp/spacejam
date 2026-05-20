//! Statistics

use std::collections::BTreeMap;

use crate::{CORES_COUNT, Ed25519Public, Extrinsic};
use serde::{Deserialize, Serialize};
use spacejson::Json;
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

/// Per-validator activity records for one epoch
pub type ValidatorStats =
    crate::Array<ValidatorActivityRecord, { crate::VALIDATORS_COUNT as usize }>;

/// Per-core activity records
pub type CoreStats = crate::Array<CoreActivityRecord, CORES_COUNT>;

/// Represents statistics.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default, Json)]
pub struct Statistics {
    /// Current epoch statistics (πV)
    #[serde(rename = "vals_curr_stats")]
    #[json(Vec<ValidatorActivityRecordJson>)]
    pub vals_current: ValidatorStats,

    /// Last epoch statistics (πL)
    #[serde(rename = "vals_last_stats")]
    #[json(Vec<ValidatorActivityRecordJson>)]
    pub vals_last: ValidatorStats,

    /// Current core activity records (πC)
    #[json(Vec<CoreActivityRecordJson>)]
    pub cores: CoreStats,

    /// Current service activity records (πS)
    pub services: BTreeMap<u32, ServiceActivityRecord>,
}

impl Statistics {
    /// Update the statistics
    pub fn update(
        &mut self,
        new_epoch: bool,
        index: u16,
        extrinsic: &Extrinsic,
    ) -> anyhow::Result<()> {
        self.services.clear();
        self.update_blocks(new_epoch, index, extrinsic)?;
        self.update_preimages(index, extrinsic);
        self.update_assurances(extrinsic);
        self.update_guarantees(extrinsic);
        Ok(())
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

    // Update DA load and popularity only for newly available reports
    pub fn merge_reports(
        &mut self,
        available: &[crate::service::WorkReport],
        assurances: &[u32; crate::CORES_COUNT],
    ) {
        for report in available {
            let core_index = report.core_index as usize;
            let core = &mut self.cores[core_index];

            // (d) DA load calculation: bundle size + segment overhead for exported segments
            let bundle_size = report.spec.length;
            let segment_overhead = (report.spec.exports_count as u64)
                .saturating_mul(65)
                .div_ceil(64)
                .saturating_mul(crate::SEGMENT_SIZE as u64);

            core.da_load += bundle_size + segment_overhead as u32;
        }

        for (core_index, &assurance_count) in assurances.iter().enumerate() {
            // (p) Popularity - total assurance count for this core
            self.cores[core_index].popularity = assurance_count as u16;
        }
    }

    /// Merge the reporter statistics
    pub fn merge_reporters(
        &mut self,
        reporters: &[Ed25519Public],
        validators: &[Ed25519Public],
    ) -> anyhow::Result<()> {
        for reporter in reporters.iter() {
            let Some(index) = validators.iter().position(|v| v == reporter) else {
                continue;
                // anyhow::bail!("reporter is invalid");
            };

            self.vals_current[index].guarantees += 1;
        }
        Ok(())
    }

    // Update validator statistics for blocks
    fn update_blocks(
        &mut self,
        new_epoch: bool,
        index: u16,
        extrinsic: &Extrinsic,
    ) -> anyhow::Result<()> {
        if new_epoch {
            self.vals_last = std::mem::take(&mut self.vals_current);
        }

        if index >= crate::VALIDATORS_COUNT {
            anyhow::bail!("author index is invalid");
        }

        self.vals_current[index as usize].blocks += 1;
        self.vals_current[index as usize].tickets += extrinsic.tickets.len() as u32;
        Ok(())
    }

    // Update validator / service statistics
    fn update_preimages(&mut self, index: u16, extrinsic: &Extrinsic) {
        let author_stats = &mut self.vals_current[index as usize];
        author_stats.pre_images += extrinsic.preimages.len() as u32;
        author_stats.pre_images_size += extrinsic
            .preimages
            .iter()
            .map(|p| p.blob.len())
            .sum::<usize>() as u32;

        // Update service statistics for provided preimages
        for preimage in &extrinsic.preimages {
            let service = self.services.entry(preimage.requester).or_default();
            service.provided_count += 1;
            service.provided_size += preimage.blob.len() as u32;
        }
    }

    // Update validator statistics for assurances
    fn update_assurances(&mut self, extrinsic: &Extrinsic) {
        for assurance in &extrinsic.assurances {
            self.vals_current[assurance.validator_index as usize].assurances += 1;
        }
    }

    // Update validator / service statistics for guarantees
    fn update_guarantees(&mut self, extrinsic: &Extrinsic) {
        for core in &mut self.cores {
            *core = CoreActivityRecord::default();
        }

        // Update core statistics from guaranteed work reports (incoming work-reports w)
        for report in &extrinsic.guarantees {
            let core_index = report.report.core_index as usize;
            let core = &mut self.cores[core_index];
            core.bundle_size += report.report.spec.length;

            // Aggregate statistics from all work results for this core
            for result in &report.report.results {
                core.imports += result.refine_load.imports;
                core.exports += result.refine_load.exports;
                core.extrinsic_size += result.refine_load.extrinsic_size;
                core.extrinsic_count += result.refine_load.extrinsic_count;
                core.gas_used += result.refine_load.gas_used;

                // Update service statistics
                let service = self.services.entry(result.service_id).or_default();
                service.refinement_count += 1;
                service.refinement_gas_used += result.refine_load.gas_used;
                service.imports += result.refine_load.imports as u32;
                service.exports += result.refine_load.exports as u32;
                service.extrinsic_count += result.refine_load.extrinsic_count as u32;
                service.extrinsic_size += result.refine_load.extrinsic_size;
            }
        }
    }
}
