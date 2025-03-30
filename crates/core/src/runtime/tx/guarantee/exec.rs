//! Execution of work reports

use crate::{
    runtime::Storage,
    service::{Privileges, WorkReport},
    Gas,
};

/// (Δ+) outer accumulation
pub fn exec(
    _gas_limit: Gas,
    _reports: Vec<WorkReport>,
    _accounts: &impl Storage,
    _privileges: &Privileges,
) {
}
