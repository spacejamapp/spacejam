//! Operand for the virtual machine

use crate::{
    runtime::vm::{DeferredTransfer, StateContext},
    service::WorkExecResult,
    Gas, OpaqueHash,
};

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
