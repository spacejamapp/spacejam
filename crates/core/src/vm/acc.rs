//! Operand for the virtual machine

use std::collections::BTreeMap;

use crate::{
    service::WorkExecResult,
    vm::{DeferredTransfer, StateContext},
    Gas, OpaqueHash, ServiceId,
};

/// The commitment map
pub type CommitmentMap = BTreeMap<ServiceId, OpaqueHash>;

/// The result of the execution
///
/// - N: the number of work-results accumulated.
/// - U: A posterior state-context.
/// - [T]: resultant deferred-transfers
/// - B: accumulation-output pairings.
/// - U: the total gas used
#[derive(Default)]
pub struct Accumulated {
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

/// The accumulate result of (ΨA)
#[derive(Default)]
pub struct AccumulateResult {
    /// (o) The state context
    pub context: StateContext,

    /// (t) The timeslot for the current accumulation
    pub transfers: Vec<DeferredTransfer>,

    /// (b) The output hash of the accumulation
    pub hash: Option<OpaqueHash>,

    /// (u) The gas used
    pub gas: Gas,
}

/// An operand of the accumulation
///
/// defined per GP (12.19)
pub struct Operand {
    /// (d) The work execution result
    pub data: WorkExecResult,

    /// (e) The erasure root
    pub erasure_root: OpaqueHash,

    /// (o) The authorizer output
    pub authorizer_output: Vec<u8>,

    /// (y) The payload blob hash
    pub payload: OpaqueHash,

    /// (h) The hash of the work package
    pub hash: OpaqueHash,

    /// (n) The accumulate gas
    pub gas: Gas,
}
