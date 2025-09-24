//! Authorize interface

use account::Accounts;
use anyhow::Result;
use pvm::Pvm;
use score::service::{PackageValidation, WorkExecResult, WorkPackage};

/// Authorize the work package
pub fn authorize<R: Accounts, VM: Pvm>(
    work: &WorkPackage,
    core_idx: u16,
    accounts: &mut R,
) -> Result<(Vec<u8>, u64)> {
    // TODO: remove the package validation instance
    let _validation = PackageValidation::new(work);
    _validation.validate()?;

    // execute is-authorized invocation (Ψ_I)
    let auth_result = VM::is_authorized(work, core_idx, accounts, work.context.lookup_anchor_slot);
    if !matches!(auth_result.exec, WorkExecResult::Ok(_)) {
        anyhow::bail!("Work package authorization failed: {:?}", auth_result.exec);
    }

    let auth_output = match auth_result.exec {
        WorkExecResult::Ok(output) => output,
        _ => Vec::new(),
    };

    Ok((auth_output, auth_result.gas))
}
