//! Guarantor abstraction

use crate::{d3l::Shard, DataLake, WorkPackageBundle};
use anyhow::Result;
use pvm::Pvm;
use score::{
    service::{WorkPackage, WorkReport},
    Accounts, CoreIndex, OpaqueHash,
};
use std::collections::BTreeMap;

mod authorize;
mod refine;

/// Guarantor abstraction
#[allow(async_fn_in_trait)]
pub trait Guarantor: DataLake + Sized {
    /// Compute work package and produce work report
    async fn compute<A: Accounts, VM: Pvm>(
        &self,
        core_idx: CoreIndex,
        extrinsic: Vec<Vec<u8>>,
        work: &WorkPackage,
        accounts: &mut A,
    ) -> Result<(WorkReport, WorkPackageBundle)> {
        let (auth_output, auth_gas_used) = authorize::authorize::<A, VM>(work, core_idx, accounts)?;
        let mut extrinsic_data = BTreeMap::new();
        for extrinsic in extrinsic {
            extrinsic_data.insert(crypto::blake2b(&extrinsic), extrinsic);
        }

        let (mut report, bundle) = refine::refine::<A, VM>(
            self,
            work,
            &extrinsic_data,
            core_idx,
            &auth_output,
            accounts,
        )
        .await?;
        report.auth_output = auth_output;
        report.auth_gas_used = auth_gas_used;

        // build the segment roots
        let mut work_package_hashes = Vec::new();
        for item in &work.items {
            for import_spec in &item.import_segments {
                if let Ok(Some(_)) = self.get_segment_root(&import_spec.tree_root).await {
                    work_package_hashes.push(import_spec.tree_root);
                }
            }
        }
        report.lookup = self.lookup(&work_package_hashes).await?;
        Ok((report, bundle))
    }

    /// Compute the work package synchronously
    fn compute_sync<A: Accounts, VM: Pvm>(
        &self,
        core_idx: CoreIndex,
        extrinsic: Vec<Vec<u8>>,
        work: &WorkPackage,
        accounts: &mut A,
    ) -> Result<(WorkReport, WorkPackageBundle)> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(self.compute::<A, VM>(core_idx, extrinsic, work, accounts))
    }

    /// Validate a work package bundle
    ///
    /// TODO: handle the segment roots
    async fn validate<A: Accounts, VM: Pvm>(
        &self,
        bundle: &WorkPackageBundle,
        core_index: CoreIndex,
        segment_roots: BTreeMap<OpaqueHash, OpaqueHash>,
        accounts: &mut A,
    ) -> Result<WorkReport> {
        let work = &bundle.package;
        let extrinsic: Vec<Vec<u8>> = bundle.extrinsic.values().cloned().collect();
        let (mut report, _) = self
            .compute::<A, VM>(core_index, extrinsic, work, accounts)
            .await?;

        report.lookup = segment_roots;
        Ok(report)
    }

    /// Get shard data with justifications
    async fn shard(&self, erasure_root: OpaqueHash, shard_index: u16) -> Result<Shard> {
        // Get the specific shard data
        let bundle_shard = self
            .get_shard(&erasure_root, shard_index)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Shard not found: {:?} index {}", erasure_root, shard_index)
            })?;

        // Get all shards to extract segment shards
        let all_shards = self.get_shards(&erasure_root).await?.ok_or_else(|| {
            anyhow::anyhow!("No shards found for erasure root: {:?}", erasure_root)
        })?;

        // Return segment shards as well as bundle shard
        // TODO: Properly separate bundle vs segment shards based on the erasure coding layout
        let segment_shards = if (shard_index as usize) < all_shards.len() {
            all_shards[shard_index as usize].clone()
        } else {
            Vec::new()
        };

        // Generate justification for the shard
        let justifications = self
            .bundle_justification(&erasure_root, shard_index)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not generate justification for shard {} of {:?}",
                    shard_index,
                    erasure_root
                )
            })?;

        Ok(Shard {
            bundle: bundle_shard,
            segment_shards,
            justifications,
        })
    }
}

impl<T> Guarantor for T where T: DataLake {}
