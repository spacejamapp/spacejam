//! Chain environment

use anyhow::{anyhow, Result};
use score::{
    block::Head,
    service::{RefineContext, ServiceAccount},
    EntropyBuffer, OpaqueHash, ServiceId,
};
use std::collections::BTreeMap;

/// Chain environment
#[derive(Clone, Default)]
pub struct Chain {
    /// Best block
    pub best: Head,

    /// Entropy buffer
    pub entropy: EntropyBuffer,

    /// Finalized block
    pub finalized: Head,

    /// Service accounts
    pub accounts: BTreeMap<u32, ServiceAccount>,
}

impl Chain {
    /// Find a service code
    pub fn service(&self, service: ServiceId) -> Result<OpaqueHash> {
        tracing::info!("service: {:?}", service);
        self.accounts
            .get(&service)
            .map(|account| account.code)
            .ok_or_else(|| anyhow!("Service not found"))
    }

    /// Get the refine context
    ///
    /// TODO: support prerequisites
    pub fn refine_context(&self) -> RefineContext {
        RefineContext {
            anchor: self.best.hash,
            state_root: Default::default(),
            beefy_root: Default::default(),
            lookup_anchor: self.finalized.hash,
            lookup_anchor_slot: self.finalized.slot,
            prerequisites: Default::default(),
        }
    }
}
