//! PolkaVM environment

use crate::{host::Accumulate, invocation::general::Received, Reason};
use score::{
    service::{ServiceAccount, WorkExecResult},
    vm::{DeferredTransfer, StateContext},
    Gas, OpaqueHash, ServiceId,
};

/// Context for the PVM accumulation
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

/// The result of is-authorized invocation (ΨI)
pub struct Executed {
    /// The output
    pub data: Vec<u8>,

    /// The reason
    pub exec: WorkExecResult,

    /// The gas used
    pub gas: Gas,
}

impl Executed {
    /// Create a new executed result
    pub fn new(data: Vec<u8>, exec: WorkExecResult, gas: Gas) -> Self {
        Self { data, exec, gas }
    }
}

impl Received<Accumulate> {
    /// Convert the received result to an accumulate result
    pub fn to_result(self) -> AccumulateResult {
        // Treat Continue and Halt as successful completion
        // Only Panic, OOG, and Fault should use Y context (exceptional dimension)
        match self.reason {
            Reason::Continue | Reason::Halt => {
                let mut result = self.data.x.to_result(self.gas);
                if self.output.len() == 32 {
                    let mut hash = [0; 32];
                    hash.copy_from_slice(&self.output);
                    result.hash = Some(hash);
                }
                result
            }
            _ => self.data.y.to_result(self.gas),
        }
    }
}
