//! work report computation

use crate::{SegmentProvider, Worker};
use anyhow::Result;
use pvm::Pvm;
use score::{
    service::{WorkPackage, WorkReport},
    Accounts, CoreIndex,
};

impl<P: SegmentProvider> Worker<P> {
    /// Compute the work package according to Gray Paper specifications with segment provider
    pub async fn compute<R: Accounts, VM: Pvm>(
        mut self,
        work: WorkPackage,
        core_idx: usize,
        mut accounts: R,
    ) -> Result<WorkReport> {
        self.authorize::<R, VM>(&work, core_idx, &mut accounts)?;
        let encoded = codec::encode(&work)?;
        self.report.spec.hash = crypto::blake2b(&encoded);
        // Erasure root will be computed during refine
        self.report.spec.length = encoded.len() as u32;
        self.report.core_index = core_idx as CoreIndex;
        self.report.authorizer_hash = work.authorizer.hash();
        self.report.lookup = Default::default();
        self.refine::<R, VM>(&work, &mut accounts, core_idx as u16)
            .await?;
        self.report.context = work.context;
        Ok(self.report)
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
