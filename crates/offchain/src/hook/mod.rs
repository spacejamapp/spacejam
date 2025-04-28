//! Hook implementation for offchain

use std::collections::HashMap;

use crate::service::Rpc;
use rpc::RpcHook;
use score::{Block, OpaqueHash};

mod rpc;

/// Hook implementation for offchain
///
/// NOTE: current we only implement the hook for rpc.
pub struct OffchainHook<C: runtime::Config> {
    rpc: RpcHook<C>,
}

impl<C: runtime::Config> OffchainHook<C> {
    /// Create a new offchain hook
    pub fn new(rpc: Rpc<C>) -> Self {
        Self {
            rpc: RpcHook::new(rpc),
        }
    }
}

impl<C: runtime::Config> runtime::Hook for OffchainHook<C> {
    async fn on_finalized_block(
        &self,
        block: Block,
        diff: HashMap<OpaqueHash, Vec<u8>>,
    ) -> anyhow::Result<()> {
        self.rpc.on_finalized_block(block, diff).await
    }
}
