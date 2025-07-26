//! authorization

use crate::Worker;
use anyhow::Result;
use pvm::Invocation;
use runtime::Config;
use score::{
    Accounts,
    service::{WorkExecResult, WorkPackage},
};

impl<C: Config> Worker<C> {
    /// Phase 1: Process authorization (validation + Is-Authorized invocation)
    pub fn authorize<R: Accounts>(
        &mut self,
        work: &WorkPackage,
        core_idx: usize,
        accounts: &mut R,
    ) -> Result<()> {
        // TODO: subscribe the validation statistics
        let _validation = work.validate()?;

        // execute is-authorized invocation (Ψ_I)
        let auth_result = C::Vm::is_authorized(
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

        self.report.auth_output = auth_output;
        self.report.auth_gas_used = auth_result.gas;
        Ok(())
    }
}
