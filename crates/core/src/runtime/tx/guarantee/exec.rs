//! Execution of work reports

use crate::{
    runtime::{vm::Vm, Storage},
    service::{Privileges, WorkReport},
    Gas,
};

/// (Δ+) outer accumulation
pub fn exec<V: Vm>(
    _gas_limit: Gas,
    _reports: Vec<WorkReport>,
    _accounts: &impl Storage,
    _privileges: &Privileges,
) {
}
