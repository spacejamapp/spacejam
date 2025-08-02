//! work report computation

use crate::{bundle::WorkPackageBundle, DataLake, Worker};
use anyhow::Result;
use pvm::Pvm;
use score::{
    service::{WorkPackage, WorkReport},
    Accounts, CoreIndex,
};

impl<S: DataLake> Worker<S> {
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
        self.authorize::<R, VM>(&work, core_idx, &mut accounts, &mut report)?;
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

        // Refine the work package
        report.lookup = self.segment_provider.lookup(&work_package_hashes).await?;
        let (spec, results) = self
            .refine::<R, VM>(&work, &mut accounts, core_idx as u16, &report.auth_output)
            .await?;
        report.spec = spec;
        report.results = results;
        report.context = work.context;
        Ok(report)
    }

    /// Compute the work package bundle according to Gray Paper specifications
    ///
    /// TODO: validate the work package bundle?
    pub async fn compute_bundle<R: Accounts, VM: Pvm>(
        self,
        bundle: WorkPackageBundle,
        core_idx: usize,
        accounts: R,
    ) -> Result<WorkReport> {
        self.compute::<R, VM>(bundle.package, core_idx, accounts)
            .await
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
