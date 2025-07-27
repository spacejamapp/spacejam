//! Execution API of JAM VM

use crate::Jam;
use anyhow::Result;
use pvm::Invocation;
use pvmi::Interpreter;
use score::service::WorkPackage;

impl Jam {
    /// Authorize the work package
    pub fn authorize(&mut self, work: &WorkPackage, core_idx: u16) -> Result<pvm::Executed> {
        Ok(Interpreter::is_authorized(
            work,
            core_idx,
            &mut self.chain.accounts,
            self.chain.best.slot,
        ))
    }

    /// Refine the work package
    pub fn refine(&mut self, _work: &WorkPackage) -> Result<pvm::Executed> {
        todo!()
    }
}
