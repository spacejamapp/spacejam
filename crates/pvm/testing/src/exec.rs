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
    ///
    /// NOTE: run refine for all work items
    pub fn refine(&mut self, work: &WorkPackage) -> Result<pvm::Refined> {
        Ok(Interpreter::refine(
            0,
            0,
            work,
            Default::default(),
            Default::default(),
            Default::default(),
            &mut self.chain.accounts,
            self.chain.best.slot,
        ))
    }

    /// Accumulate the work package
    ///
    /// 1. convert work package to work report
    /// 2. run accumulate for all work items
    /// 3. return the accumulated result
    pub fn accumulate(&mut self, _work: &WorkPackage) -> Result<pvm::Executed> {
        todo!()
    }
}
