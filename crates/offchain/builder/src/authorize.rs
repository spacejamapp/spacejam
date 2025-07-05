//! authorization

use crate::Worker;
use anyhow::Result;
use pvm::Invocation;
use runtime::Config;
use score::{
    service::{WorkExecResult, WorkPackage},
    Account, Accounts,
};

impl<'a, C: Config> Worker<'a, C> {
    /// Phase 1: Process authorization (validation + Is-Authorized invocation)
    pub fn authorize<R: Accounts>(
        &mut self,
        work: &WorkPackage,
        core_idx: usize,
        accounts: &mut R,
    ) -> Result<()> {
        // TODO: subscribe the validation statistics
        let _validation = work.validate()?;
        let Some(auth_account) = accounts.get(work.auth_code_host) else {
            anyhow::bail!(
                "Authorization code host service {} not found",
                work.auth_code_host
            );
        };

        // historical lookup for authorization code (Λ function)
        let code = auth_account
            .historical_lookup(work.context.lookup_anchor_slot, work.authorizer.code_hash)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Authorization code with hash {:?} not available at timeslot {}",
                    work.authorizer.code_hash,
                    work.context.lookup_anchor_slot
                )
            })?;

        // execute is-authorized invocation (Ψ_I)
        let auth_result = C::Vm::is_authorized(&code, core_idx as u16);
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
