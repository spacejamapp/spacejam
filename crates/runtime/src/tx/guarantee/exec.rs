//! Execution of work reports

use crate::Storage;
use pvm::Pvm;
use score::{
    Gas, ServiceId,
    service::WorkReport,
    vm::{Accumulated, StateContext},
};
use std::collections::BTreeMap;

/// (Δ+) outer accumulation
///
/// (N_G, [W], U, D(N_S -> N_G)) -> (N, U, [T], B, U)
///
/// parameters:
/// - N_G: gas limit
/// - [W]: work reports
/// - U: state context
/// - D(N_S -> N_G): gas table
///
/// returns:
/// - N: the number of work-results accumulated.
/// - U: A posterior state-context.
/// - [T]: resultant deferred-transfers
/// - B: accumulation-output pairings.
/// - U: the total gas used
pub fn exec<V: Pvm>(
    _gas_limit: Gas,
    _reports: Vec<WorkReport>,
    context: StateContext,
    _accounts: &impl Storage,
    _gas_table: &BTreeMap<ServiceId, Gas>,
) -> Accumulated {
    let _ = V::accumulate(context, 0, 0, 0, Default::default());
    Default::default()
}
