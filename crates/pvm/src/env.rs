//! PolkaVM environment

use crate::result::AccumulateResult;
use score::{
    service::ServiceAccount,
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
