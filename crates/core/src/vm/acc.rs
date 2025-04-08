//! Operand for the virtual machine

use serde::{Deserialize, Serialize};

use crate::{
    service::{ServiceAccount, WorkExecResult},
    vm::{DeferredTransfer, StateContext},
    Gas, OpaqueHash, ServiceId,
};
use std::collections::BTreeMap;

/// The commitment map
pub type CommitmentMap = BTreeMap<ServiceId, OpaqueHash>;

/// The result of the execution
///
/// - N: the number of work-results accumulated.
/// - U: A posterior state-context.
/// - [T]: resultant deferred-transfers
/// - B: accumulation-output pairings.
/// - U: the total gas used
#[derive(Default, Clone)]
pub struct Accumulated {
    /// (i) the number of work-results accumulated.
    pub accumulated: usize,

    /// (o) A posterior state-context.
    pub context: StateContext,

    /// (t) The resultant deferred-transfers
    pub transfers: Vec<DeferredTransfer>,

    /// (b) The accumulation-output pairings.
    pub pairings: CommitmentMap,

    /// (u) The total gas used
    pub gas: BTreeMap<ServiceId, Gas>,
}

/// Context for the accumulation
#[derive(Default, Clone)]
pub struct AccumulateContext {
    /// (s) The service id
    pub service: ServiceId,

    /// (u) The upcoming validators
    pub context: StateContext,

    /// (i) empty index for a new account
    pub index: ServiceId,

    /// (t) The deferred transfer
    pub transfer: Vec<DeferredTransfer>,

    /// (y) The output hash of the accumulation
    pub output: Option<OpaqueHash>,
}

impl AccumulateContext {
    /// Get the account for the accumulation
    pub fn account(&mut self) -> Option<&mut ServiceAccount> {
        self.context.accounts.get_mut(&self.service)
    }

    /// Check update an empty account index
    pub fn check(&mut self, index: ServiceId) {
        if !self.context.accounts.contains_key(&index) {
            self.index = index;
        } else {
            let next = ((index - (1 << 8)) + 1) % (u32::MAX - (1 << 9)) + (1 << 8);
            self.check(next);
        }
    }

    /// Convert the accumulate context to an accumulate result
    pub fn to_result(self, gas: Gas) -> AccumulateResult {
        AccumulateResult {
            context: self.context,
            transfers: self.transfer,
            hash: self.output,
            gas,
        }
    }
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
#[derive(Serialize, Deserialize)]
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
