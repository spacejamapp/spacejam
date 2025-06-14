//! PolkaVM environment

use crate::{host::Accumulate, invocation::state::Received, Reason};
use score::{
    account::{Account, Accounts},
    vm::{DeferredTransfer, StateContext},
    Gas, OpaqueHash, ServiceId, TimeSlot,
};

/// Context for the PVM accumulation
#[derive(Clone)]
pub struct AccumulateContext<R: Accounts> {
    /// (s) The service id
    pub service: ServiceId,

    /// (u) The upcoming validators
    pub context: StateContext<R>,

    /// (i) empty index for a new account
    pub index: ServiceId,

    /// (t) The deferred transfer
    pub transfer: Vec<DeferredTransfer>,

    /// (y) The output hash of the accumulation
    pub output: Option<OpaqueHash>,
}

impl<R: Accounts> AccumulateContext<R> {
    /// Get the account for the accumulation
    pub fn account(&mut self) -> Option<&mut (impl Account + '_)> {
        self.context.accounts.get(self.service)
    }

    /// Check update an empty account index
    ///
    /// TODO: use a loop instead of recursion
    pub fn check(&mut self, index: ServiceId) {
        let services = self.context.accounts.services();
        if !services.contains(&index) {
            self.index = index;
        } else {
            let next = ((index - (1 << 8)) + 1) % (u32::MAX - (1 << 9)) + (1 << 8);
            self.check(next);
        }
    }

    /// Convert the accumulate context to an accumulate
    pub fn accumulate(self, timeslot: TimeSlot) -> Accumulate<R> {
        Accumulate {
            y: self.clone(),
            x: self,
            timeslot,
        }
    }

    /// Convert the accumulate context to an accumulate result
    pub fn to_result(self, gas: Gas) -> AccumulateResult<R> {
        AccumulateResult {
            context: self.context,
            transfers: self.transfer,
            hash: self.output,
            gas,
        }
    }
}

/// The accumulate result of (ΨA)
pub struct AccumulateResult<R: Accounts> {
    /// (o) The state context
    pub context: StateContext<R>,

    /// (t) The timeslot for the current accumulation
    pub transfers: Vec<DeferredTransfer>,

    /// (b) The output hash of the accumulation
    pub hash: Option<OpaqueHash>,

    /// (u) The gas used
    pub gas: Gas,
}

impl<R: Accounts> AccumulateResult<R> {
    /// Create a new accumulate result
    pub fn new(context: StateContext<R>) -> Self {
        Self {
            context,
            transfers: Vec::new(),
            hash: None,
            gas: 0,
        }
    }
}

impl<R: Accounts> Received<Accumulate<R>> {
    /// Convert the received result to an accumulate result
    pub fn to_result(self) -> AccumulateResult<R> {
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
