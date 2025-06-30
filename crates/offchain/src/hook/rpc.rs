//! RPC hook implementation

use crate::service::Rpc;
use rpc::{
    core::server::SubscriptionMessage,
    server::{ServicePreimageFilter, ServiceRequestFilter, ServiceValueFilter},
};
use runtime::storage::{KVStorage, Storage, SyncStorage};
use score::{block::Head, state::key, Block, OpaqueHash, ServiceId, TrieKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
};

/// The RPC hook
pub struct RpcHook<C: runtime::Config> {
    rpc: Rpc<C>,
}

impl<C: runtime::Config> RpcHook<C> {
    /// Create a new offchain hook
    pub fn new(rpc: Rpc<C>) -> Self {
        Self { rpc }
    }

    /// Migrate the parent
    fn migrate_parent(
        &self,
        _hash: &OpaqueHash,
        parent: &OpaqueHash,
        state_root: &OpaqueHash,
    ) -> anyhow::Result<()> {
        let parent = self.runtime.storage.block(parent)?;
        let head = parent.header.clone();

        self.runtime.storage.set_header(&head)?;
        self.runtime
            .storage
            .set_state_root(&head.parent, state_root)?;
        Ok(())
    }

    /// Migrate the statistics
    async fn migrate_statistics(&self, hash: &OpaqueHash) -> anyhow::Result<()> {
        let Some(statistics) = self.runtime.storage.get(key::STATISTICS)? else {
            return Ok(());
        };

        // 1. set the statistics
        let key = [hash.as_ref(), key::STATISTICS.as_ref()].concat();
        self.runtime.storage.set(key, statistics.clone())?;

        // 2. subscribe the statistics
        self.dispatch_statistics(&statistics).await
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
        self.runtime.storage.set(key, beefy_root)?;
        Ok(())
    }
}

impl<C: runtime::Config> runtime::Hook for RpcHook<C> {
    // NOTE: since grandpa is not fully implemented, we set the best block
    // together with the finalized block.
    async fn on_finalized_block(&self, block: Block) -> anyhow::Result<()> {
        let header = block.header.clone();
        self.runtime.storage.set_header(&header)?;

        let head = Head {
            hash: header.hash()?,
            slot: header.slot,
        };
        self.runtime.storage.finalize(&head)?;

        // 1. dispatch the best and finalized block
        self.dispatch_best_block(&head.hash, head.slot as u64)
            .await?;
        self.dispatch_finalized_block(&head.hash, head.slot as u64)
            .await?;

        // 2. migrate states
        self.migrate_statistics(&head.hash).await?;
        self.migrate_beefy_root(&head.hash).await?;
        self.migrate_parent(
            &head.hash,
            &block.header.parent,
            &block.header.parent_state_root,
        )?;

        Ok(())
    }

    async fn on_service_data(
        &self,
        hash: OpaqueHash,
        data: BTreeMap<ServiceId, Vec<u8>>,
    ) -> anyhow::Result<()> {
        // update the service list
        if !data.is_empty() {
            let key = [hash.as_ref(), b"services"].concat();
            let mut plist: BTreeSet<u32> =
                codec::decode(&self.runtime.storage.get(&key)?.unwrap_or_default())?;
            plist.extend(data.keys().copied());
            self.runtime.storage.set(key, codec::encode(&plist)?)?;
        }

        for (service, sink) in self.service_data_sub.lock().await.iter() {
            if let Some(value) = data.get(service) {
                sink.send(SubscriptionMessage::from_json(value)?).await?;
            }
        }
        Ok(())
    }

    async fn on_service_value(
        &self,
        _hash: OpaqueHash,
        data: BTreeMap<ServiceId, (Vec<u8>, Vec<u8>)>,
    ) -> anyhow::Result<()> {
        for (service, sink) in self.service_value_sub.lock().await.iter() {
            let ServiceValueFilter { service, key } = service;
            if let Some((skey, value)) = data.get(service) {
                if skey[8..] != key[..24] {
                    continue;
                }

                sink.send(SubscriptionMessage::from_json(value)?).await?;
            }
        }
        Ok(())
    }

    async fn on_service_preimage(
        &self,
        _hash: OpaqueHash,
        data: BTreeMap<ServiceId, (Vec<u8>, Vec<u8>)>,
    ) -> anyhow::Result<()> {
        for (service, sink) in self.service_preimage_sub.lock().await.iter() {
            let ServicePreimageFilter { service, hash } = service;
            if let Some((skey, value)) = data.get(service) {
                if skey[8..] != hash[..24] {
                    continue;
                }

                sink.send(SubscriptionMessage::from_json(value)?).await?;
            }
        }
        Ok(())
    }

    async fn on_service_request(
        &self,
        _hash: OpaqueHash,
        data: BTreeMap<ServiceId, (u32, Vec<u8>, Vec<u8>)>,
    ) -> anyhow::Result<()> {
        for (service, sink) in self.service_request_sub.lock().await.iter() {
            let ServiceRequestFilter {
                service,
                hash,
                length,
            } = service;
            if let Some((len, skey, value)) = data.get(service) {
                // TODO: shall we check the length here?
                if skey[8..] != hash[..24] || len != length {
                    continue;
                }

                sink.send(SubscriptionMessage::from_json(value)?).await?;
            }
        }

        Ok(())
    }

    fn on_key_value(&self, hash: OpaqueHash, key: TrieKey, value: &[u8]) -> anyhow::Result<()> {
        let bkey = [hash.as_ref(), key.as_ref()].concat();
        self.runtime.storage.set(bkey, value)?;
        Ok(())
    }
}

impl<C: runtime::Config> Deref for RpcHook<C> {
    type Target = Rpc<C>;

    fn deref(&self) -> &Self::Target {
        &self.rpc
    }
}
