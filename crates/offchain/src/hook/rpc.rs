//! RPC hook implementation

use crate::service::Rpc;
use rpc::{
    core::server::SubscriptionMessage,
    server::{ServicePreimageFilter, ServiceRequestFilter, ServiceValueFilter},
};
use runtime::storage::StateStorage;
use score::{state::key, Block, OpaqueHash, ServiceId};
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
        _parent: &OpaqueHash,
        _state_root: &OpaqueHash,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// Migrate the statistics
    async fn migrate_statistics(&self, hash: &OpaqueHash) -> anyhow::Result<()> {
        let chain = self.runtime.chain().await;
        let best = chain.best_chain()?;
        let Some(statistics) = best.state.state_get(key::STATISTICS)? else {
            return Ok(());
        };

        // 1. set the statistics
        let key = [hash.as_ref(), key::STATISTICS.as_ref()].concat();
        best.state.state_set(key, statistics.clone())?;

        // 2. subscribe the statistics
        self.dispatch_statistics(&statistics).await
    }

    /// Migrate the beefy root
    ///
    /// TODO: make the beefy root key constant somewhere
    async fn migrate_beefy_root(&self, hash: &OpaqueHash) -> anyhow::Result<()> {
        let chain = self.runtime.chain().await;
        let best = chain.best_chain()?;
        let history = best.state.recent_blocks()?;
        let Some(block) = history.last() else {
            return Ok(());
        };

        if block.header_hash != *hash {
            return Ok(());
        }

        /*   let Some(beefy_root) = block.mmr.root() else {
            return Ok(());
        }; */

        // let key = [hash.as_ref(), b"beefy_root"].concat();
        // best.state.sync_set(key, beefy_root)?;
        Ok(())
    }
}

impl<C: runtime::Config> runtime::Hook for RpcHook<C> {
    // NOTE: since grandpa is not fully implemented, we set the best block
    // together with the finalized block.
    async fn on_finalized_block(&self, block: Block) -> anyhow::Result<()> {
        let head = block.header.head()?;

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
        let chain = self.runtime.chain().await;
        let best = chain.best_chain()?;

        // update the service list
        if !data.is_empty() {
            let key = [hash.as_ref(), b"services"].concat();
            let mut plist: BTreeSet<u32> =
                codec::decode(&best.state.state_get(&key)?.unwrap_or_default())?;
            plist.extend(data.keys().copied());
            best.state.state_set(key, codec::encode(&plist)?)?;
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
}

impl<C: runtime::Config> Deref for RpcHook<C> {
    type Target = Rpc<C>;

    fn deref(&self) -> &Self::Target {
        &self.rpc
    }
}
