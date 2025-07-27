//! Execution API of JAM VM

use crate::Jam;
use anyhow::Result;
use score::service::WorkPackage;

impl Jam {
    /// Authorize the work package
    pub fn authorize(&mut self, work: &WorkPackage, core_idx: u16) -> Result<pvm::Executed> {
        self.chain.authorize(work, core_idx)
    }
}
