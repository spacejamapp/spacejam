//! Hook implementation for offchain

use crate::service::Rpc;
use std::ops::Deref;

/// Hook implementation for offchain
///
/// NOTE: current we only implement the hook for rpc.
pub struct OffchainHook<C: runtime::Config>(Rpc<C>);

impl<C: runtime::Config> OffchainHook<C> {
    /// Create a new offchain hook
    pub fn new(rpc: Rpc<C>) -> Self {
        Self(rpc)
    }
}

impl<C: runtime::Config> runtime::Hook for OffchainHook<C> {}

impl<C: runtime::Config> Deref for OffchainHook<C> {
    type Target = Rpc<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
