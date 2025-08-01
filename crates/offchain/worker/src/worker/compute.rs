//! work report computation

use crate::{SegmentProvider, WorkPackageBundle, Worker};
use anyhow::Result;
use pvm::Pvm;
use score::{
    service::{WorkPackage, WorkReport},
    Accounts, CoreIndex,
};

impl<S: SegmentProvider> Worker<S> {
    /// Compute the work package according to Gray Paper specifications
    ///
    /// Note: In the refined architecture, networking (CE133, CE135) is handled
    /// by the Network library calling Runtime methods directly.
    pub async fn compute<R: Accounts, VM: Pvm>(
        mut self,
        work: WorkPackage,
        core_idx: usize,
        mut accounts: R,
    ) -> Result<WorkReport> {
        let mut report = WorkReport::default();
        let encoded = codec::encode(&work)?;
        self.authorize::<R, VM>(&work, core_idx, &mut accounts, &mut report)?;
        report.spec.hash = crypto::blake2b(&encoded);
        report.spec.length = encoded.len() as u32;
        report.core_index = core_idx as CoreIndex;
        report.authorizer_hash = work.authorizer.hash();

        // Build segment root lookup by checking if any imports are work-package hashes
        let mut work_package_hashes = Vec::new();
        for item in &work.items {
            for import_spec in &item.import_segments {
                // Check if this tree_root is a known work-package hash
                if let Ok(Some(_)) = self
                    .segment_provider
                    .segment_root(&import_spec.tree_root)
                    .await
                {
                    work_package_hashes.push(import_spec.tree_root);
                }
            }
        }

        report.lookup = self.segment_provider.lookup(&work_package_hashes).await?;
        self.refine::<R, VM>(&work, &mut accounts, core_idx as u16, &mut report)
            .await?;
        report.context = work.context;

        Ok(report)
    }

    /// Compute the work package bundle according to Gray Paper specifications
    pub async fn compute_bundle<R: Accounts, VM: Pvm>(
        mut self,
        bundle: WorkPackageBundle,
        core_idx: usize,
        mut accounts: R,
    ) -> Result<WorkReport> {
        let work = &bundle.package;

        // Register work-package mappings with the segment provider
        for (&work_package_hash, &segment_root) in &bundle.segment_roots {
            self.segment_provider
                .register_work_package(work_package_hash, segment_root)
                .await?;
        }

        let encoded = codec::encode(work)?;
        let mut report = WorkReport::default();
        self.authorize::<R, VM>(work, core_idx, &mut accounts, &mut report)?;
        report.spec.hash = crypto::blake2b(&encoded);
        report.spec.length = encoded.len() as u32;
        report.core_index = core_idx as CoreIndex;
        report.authorizer_hash = work.authorizer.hash();

        // Build segment root lookup using the segment provider
        let work_package_hashes: Vec<_> = bundle.segment_roots.keys().copied().collect();
        report.lookup = self.segment_provider.lookup(&work_package_hashes).await?;

        self.extrinsic_data = bundle.extrinsic;
        self.refine::<R, VM>(work, &mut accounts, core_idx as u16, &mut report)
            .await?;
        report.context = work.context.clone();

        Ok(report)
    }

    /// Legacy compute method for backward compatibility
    pub fn compute_sync<R: Accounts, VM: Pvm>(
        self,
        work: WorkPackage,
        core_idx: usize,
        accounts: R,
    ) -> Result<WorkReport> {
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(self.compute::<R, VM>(work, core_idx, accounts))
    }
}
