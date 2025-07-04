//! Work package computation

use crate::{Config, Runtime};
use anyhow::Result;
use pvm::{Account, Invocation};
use score::service::{WorkPackage, WorkReport};

impl<C: Config> Runtime<C> {
    /// Compute the work package
    pub fn compute<R: score::Accounts>(
        work: WorkPackage,
        core_idx: usize,
        mut accounts: R,
    ) -> Result<WorkReport> {
        // TODO: pass this to hook.
        let _validation = work.validate()?;
        let Some(account) = accounts.get(work.auth_code_host) else {
            anyhow::bail!(
                "Authorization code host service {} not found",
                work.auth_code_host
            );
        };

        let Some(code) = account.preimage(work.authorizer.code_hash) else {
            anyhow::bail!(
                "Authorization code with hash {:?} not found",
                work.authorizer.code_hash
            );
        };

        // Execute the is-authorized invocation
        let result = C::Vm::is_authorized(&code, core_idx as u16);
        if !matches!(result.exec, score::service::WorkExecResult::Ok(_)) {
            anyhow::bail!("Work package authorization failed: {:?}", result.exec);
        }

        // Step 4: TODO - Implement remaining work report computation steps:
        // - Segment import/export handling
        // - Refine invocation for each work item
        // - Item-to-digest conversion
        // - Availability specifier generation
        // - Work report construction

        todo!("Complete work report computation implementation")
    }
}
