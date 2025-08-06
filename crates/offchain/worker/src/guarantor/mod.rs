//! Guarantor abstraction

use crate::{bundle::WorkPackageBundle, d3l::Shard, DataLake};
use anyhow::Result;
use pvm::Pvm;
use score::{
    extrinsic::{ReportGuarantee, ValidatorSignature},
    service::{WorkPackage, WorkReport},
    Accounts, CoreIndex, OpaqueHash, TimeSlot,
};
use std::collections::BTreeMap;

mod authorize;
mod refine;

/// Guarantor abstraction
#[allow(async_fn_in_trait)]
pub trait Guarantor: DataLake + Sized {
    /// On receiving a work package submission (CE133)
    ///
    /// Builder -> Guarantor
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
                if let Ok(Some(_)) = self.segment_root(&import_spec.tree_root).await {
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

    /// Validate a work package bundle (CE134)
    ///
    /// Guarantor -> Guarantor
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

    /// Create guaranteed work report (CE135)
    ///
    /// Guarantor -> Validator
    async fn guarantee(
        &self,
        _report: &WorkReport,
        _slot: TimeSlot,
        _singatures: Vec<ValidatorSignature>,
    ) -> Result<ReportGuarantee> {
        todo!()
    }

    /// On shard requests (CE137)
    async fn shard(&self, _erasure_root: OpaqueHash, _shard_index: u16) -> Result<Shard> {
        todo!()
    }
}

impl<T> Guarantor for T where T: DataLake {}
