//! Hook implementation for offchain

use crate::service::Rpc;
use runtime::{
    storage::{KVStorage, SyncStorage},
    Storage,
};
use score::{state::key, Block, OpaqueHash, ServiceId};
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

    /// Migrate the parent
    fn migrate_parent(
        &self,
        hash: &OpaqueHash,
        parent: &OpaqueHash,
        state_root: &OpaqueHash,
    ) -> anyhow::Result<()> {
        let parent = self.runtime.storage.get_block(parent)?;
        let head = parent.header.clone().try_into()?;

        self.runtime.storage.set_parent(hash, &head)?;
        self.runtime
            .storage
            .set_state_root(&head.hash, state_root)?;
        Ok(())
    }

    /// Migrate the statistics
    async fn migrate_statistics(&self, hash: &OpaqueHash) -> anyhow::Result<()> {
        let Some(statistics) = self.runtime.storage.get(key::STATISTICS)? else {
            return Ok(());
        };

        // 1. set the statistics
        let key = [hash.as_ref(), key::STATISTICS.as_ref()].concat();
        self.runtime.storage.set(&key, &statistics)?;

        // 2. subscribe the statistics
        self.0.dispatch_statistics(&statistics).await
    }

    /// Migrate the beefy root
    ///
    /// TODO: make the beefy root key constant somewhere
    async fn migrate_beefy_root(&self, hash: &OpaqueHash) -> anyhow::Result<()> {
        let history = self.runtime.storage.recent_blocks()?;
        let Some(block) = history.last() else {
            return Ok(());
        };

        if block.header_hash != *hash {
            return Ok(());
        }

        let Some(beefy_root) = block.mmr.root() else {
            return Ok(());
        };

        let key = [hash.as_ref(), b"beefy_root"].concat();
        self.runtime.storage.set(&key, beefy_root)?;
        Ok(())
    }

    async fn migrate_services(&self, hash: &OpaqueHash) -> anyhow::Result<()> {
        let services = self.runtime.storage.prefix_iter(&[255])?;
        let list = services
            .filter_map(|pair| {
                if let Ok((key, _)) = pair {
                    if key.len() != 9 {
                        return None;
                    }
                    let mut buffer = [0u8; 4];
                    buffer[0] = key[1];
                    buffer[1] = key[3];
                    buffer[2] = key[5];
                    buffer[3] = key[7];
                    Some(ServiceId::from_le_bytes(buffer))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let key = [hash.as_ref(), b"services"].concat();
        self.runtime.storage.set(&key, &codec::encode(&list)?)
    }
}

impl<C: runtime::Config> runtime::Hook for OffchainHook<C> {
    // NOTE: since grandpa is not fully implemented, we set the best block
    // together with the finalized block.
    async fn on_finalized_block(&self, block: Block) -> anyhow::Result<()> {
        let head = block.header.clone().try_into()?;
        self.runtime.storage.set_best(&head)?;
        self.runtime.storage.set_finalized(&head)?;

        // 1. dispatch the best and finalized block
        self.0
            .dispatch_best_block(&head.hash, head.slot as u64)
            .await?;
        self.0
            .dispatch_finalized_block(&head.hash, head.slot as u64)
            .await?;

        // 2. migrate states
        self.migrate_services(&head.hash).await?;
        self.migrate_statistics(&head.hash).await?;
        self.migrate_beefy_root(&head.hash).await?;
        self.migrate_parent(
            &head.hash,
            &block.header.parent,
            &block.header.parent_state_root,
        )?;

        Ok(())
    }
}

impl<C: runtime::Config> Deref for OffchainHook<C> {
    type Target = Rpc<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
