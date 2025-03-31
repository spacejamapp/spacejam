//! Execution of work reports

use std::collections::BTreeMap;

use crate::{
    runtime::{
        vm::{CommitmentMap, DeferredTransfer, StateContext, Vm},
        Storage,
    },
    service::WorkReport,
    Gas, ServiceId,
};

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
pub fn exec<V: Vm>(
    _gas_limit: Gas,
    _reports: Vec<WorkReport>,
    context: StateContext,
    _accounts: &impl Storage,
    _gas_table: &BTreeMap<ServiceId, Gas>,
) -> ExecResult {
    let _ = V::accumulate(context, 0, 0, 0, Default::default());
    Default::default()
}

/// The result of the execution
///
/// - N: the number of work-results accumulated.
/// - U: A posterior state-context.
/// - [T]: resultant deferred-transfers
/// - B: accumulation-output pairings.
/// - U: the total gas used
#[derive(Default)]
pub struct ExecResult {
    /// the number of work-results accumulated.
    pub accumulated: usize,

    /// A posterior state-context.
    pub context: StateContext,

    /// The resultant deferred-transfers
    pub transfers: Vec<DeferredTransfer>,

    /// The accumulation-output pairings.
    pub pairings: CommitmentMap,

    /// The total gas used
    pub gas: Gas,
}
