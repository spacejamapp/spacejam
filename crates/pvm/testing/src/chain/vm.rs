//! VM related stuffs

use crate::Chain;
use anyhow::Result;
use pvm::Invocation;
use pvmi::Interpreter;
use score::service::WorkPackage;

impl Chain {
    /// Authorize the work package
    pub fn authorize(&mut self, work: &WorkPackage, core_idx: u16) -> Result<pvm::Executed> {
        let result = Interpreter::is_authorized(work, core_idx, &mut self.accounts, self.best.slot);
        Ok(result)
    }
}
