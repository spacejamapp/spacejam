//! Hook implementation for offchain

use crate::service::Rpc;
use rpc::RpcHook;
use runtime::storage::Commit;
use score::{Block, OpaqueHash, TrieKey};

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
    async fn on_finalized_block(&self, block: Block) -> anyhow::Result<()> {
        self.rpc.on_finalized_block(block).await
    }

    async fn on_diff(
        &self,
        hash: OpaqueHash,
        diff: Commit<TrieKey, Vec<u8>>,
    ) -> anyhow::Result<()> {
        self.rpc.on_diff(hash, diff).await
    }
}
