//! The virtual machine interfaces of SpaceJam

use crate::{service::WorkExecResult, Gas, OpaqueHash, ServiceId, TimeSlot};
pub use context::StateContext;
use std::collections::BTreeMap;

mod context;

/// Service to hash commitment map
pub type CommitmentMap = BTreeMap<ServiceId, OpaqueHash>;

/// The virtual machine interface
pub trait Vm {
    /// (ΨA): single step state transition function
    fn accumulate(
        // (U) The state context
        _context: StateContext,
        // (N_t)  timeslot for the current accumulation
        _slot: TimeSlot,
        // (N_s)  the service id of the caller
        _service_id: ServiceId,
        // (N_g)  the gas limit for the current operation
        _gas_limit: Gas,
        // (O)  the accumulation operands
        _operands: Vec<Operand>,
    ) -> anyhow::Result<AccumulateResult>;
}

/// A deferred transfer item
pub struct DeferredTransfer {
    /// (s) The sender
    pub sender: ServiceId,

    /// (d) The destination
    pub recipient: ServiceId,

    /// (a) The amount
    pub amount: u64,

    /// (m) The memo
    pub memo: Vec<u8>,

    /// (g) The gas limit
    pub gas_limit: Gas,
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

/// The accumulate result of (ΨA)
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
