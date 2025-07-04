//! work report computation

use crate::{worker::Worker, Config};
use anyhow::Result;
use score::{
    service::{WorkPackage, WorkReport},
    CoreIndex, ErasureRoot,
};

impl<'a, C: Config> Worker<'a, C> {
    /// Compute the work package according to Gray Paper specifications
    pub fn compute<R: score::Accounts>(
        mut self,
        work: WorkPackage,
        core_idx: usize,
        mut accounts: R,
    ) -> Result<WorkReport> {
        self.authorize(&work, core_idx, &mut accounts)?;
        self.refine(&work, &mut accounts)?;
        self.report(work, core_idx)
    }

    /// Phase 4: Build the final work report
    fn report(mut self, work: WorkPackage, core_idx: usize) -> Result<WorkReport> {
        let encoded = codec::encode(&work)?;
        self.report.spec.hash = crypto::blake2b(&encoded);
        self.report.spec.erasure_root = self.erasure_root(&work)?;
        self.report.spec.length = encoded.len() as u32;
        self.report.context = work.context;
        self.report.core_index = core_idx as CoreIndex;
        self.report.authorizer_hash = work.authorizer.hash();
        self.report.lookup = vec![];
        Ok(self.report)
    }

    /// Compute erasure root for work package
    fn erasure_root(&self, _work: &WorkPackage) -> Result<ErasureRoot> {
        // TODO: Implement erasure root computation
        // This would involve erasure coding the work package bundle
        Ok([0u8; 32])
    }
}
