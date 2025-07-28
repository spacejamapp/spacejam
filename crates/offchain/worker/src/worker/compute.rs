//! work report computation

use crate::Worker;
use anyhow::Result;
use runtime::Config;
use score::{
    service::{WorkPackage, WorkReport},
    CoreIndex, ErasureRoot,
};

impl<C: Config> Worker<C> {
    /// Compute the work package according to Gray Paper specifications
    pub fn compute<R: score::Accounts>(
        mut self,
        work: WorkPackage,
        core_idx: usize,
        mut accounts: R,
    ) -> Result<WorkReport> {
        self.authorize(&work, core_idx, &mut accounts)?;
        let encoded = codec::encode(&work)?;
        self.report.spec.hash = crypto::blake2b(&encoded);
        self.report.spec.erasure_root = self.erasure_root(&work)?;
        self.report.spec.length = encoded.len() as u32;
        self.report.core_index = core_idx as CoreIndex;
        self.report.authorizer_hash = work.authorizer.hash();
        self.report.lookup = vec![];

        self.refine(&work, &mut accounts, core_idx as u16)?;
        self.report.context = work.context;
        Ok(self.report)
    }

    /// Compute erasure root for work package
    fn erasure_root(&self, _work: &WorkPackage) -> Result<ErasureRoot> {
        // TODO: Implement erasure root computation
        // This would involve erasure coding the work package bundle
        Ok([0u8; 32])
    }
}
