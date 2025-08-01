//! authorization

use crate::{SegmentProvider, Worker};
use anyhow::Result;
use pvm::Pvm;
use score::{
    service::{WorkExecResult, WorkPackage, WorkReport},
    Accounts,
};

impl<P: SegmentProvider> Worker<P> {
    /// Phase 1: Process authorization (validation + Is-Authorized invocation)
    pub fn authorize<R: Accounts, VM: Pvm>(
        &mut self,
        work: &WorkPackage,
        core_idx: usize,
        accounts: &mut R,
        report: &mut WorkReport,
    ) -> Result<()> {
        // TODO: subscribe the validation statistics
        let _validation = work.validate()?;

        // execute is-authorized invocation (Ψ_I)
        let auth_result = VM::is_authorized(
            work,
            core_idx as u16,
            accounts,
            work.context.lookup_anchor_slot,
        );
        if !matches!(auth_result.exec, WorkExecResult::Ok(_)) {
            anyhow::bail!("Work package authorization failed: {:?}", auth_result.exec);
        }

        let auth_output = match auth_result.exec {
            WorkExecResult::Ok(output) => output,
            _ => Vec::new(),
        };

        report.auth_output = auth_output;
        report.auth_gas_used = auth_result.gas;
        Ok(())
    }
}
