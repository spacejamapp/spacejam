//! RPC hook implementation

use crate::service::Rpc;
use rpc::{
    core::server::SubscriptionMessage,
    server::{ServicePreimageFilter, ServiceRequestFilter, ServiceValueFilter},
};
use runtime::storage::{KVStorage, Storage, SyncStorage};
use score::{state::key, Block, OpaqueHash, ServiceId};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
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
        self.runtime.storage.set(&key, beefy_root)?;
        Ok(())
    }

    // migrate services on state diff
    async fn migrate_services(
        &self,
        hash: &OpaqueHash,
        diff: HashMap<OpaqueHash, Vec<u8>>,
    ) -> anyhow::Result<()> {
        let mut data = BTreeMap::new();
        let mut preimage = BTreeMap::new();
        let mut request = BTreeMap::new();
        let mut svalue = BTreeMap::new();

        for (key, value) in diff {
            // skip the key that is not related to service
            if key[1..].iter().all(|b| *b == 0) {
                continue;
            }

            // service info storage
            if key[8..].iter().all(|b| *b == 0) {
                let mut service = [0u8; 4];
                service[0] = key[1];
                service[1] = key[3];
                service[2] = key[5];
                service[3] = key[7];
                data.insert(ServiceId::from_le_bytes(service), value);
                continue;
            }

            // get the service id
            let service = {
                let mut sbuf = [0u8; 4];
                sbuf[0] = key[0];
                sbuf[1] = key[2];
                sbuf[2] = key[4];
                sbuf[3] = key[6];

                ServiceId::from_le_bytes(sbuf)
            };

            let prefix = {
                let mut pbuf = [0u8; 4];
                pbuf[0] = key[1];
                pbuf[1] = key[3];
                pbuf[2] = key[5];
                pbuf[3] = key[7];
                pbuf
            };

            match prefix {
                key::ACCOUNT_STORAGE_PREFIX => {
                    svalue.insert(service, (key.to_vec(), value));
                }
                key::ACCOUNT_PREIMAGE_PREFIX => {
                    preimage.insert(service, (key.to_vec(), value));
                }

                length => {
                    let length = u32::from_le_bytes(length);
                    request.insert(service, (length, key.to_vec(), value));
                }
            }
        }

        // update the service list
        if !data.is_empty() {
            let key = [hash.as_ref(), b"services"].concat();
            let mut plist: BTreeSet<u32> =
                codec::decode(&self.runtime.storage.get(&key)?.unwrap_or_default())?;
            plist.extend(data.keys().copied());
            self.runtime.storage.set(&key, &codec::encode(&plist)?)?;
        }

        tokio::try_join!(
            self.migrate_service_data(data),
            self.migrate_service_value(svalue),
            self.migrate_service_preimage(preimage),
            self.migrate_service_request(request)
        )?;

        Ok(())
    }

    /// Migrate the service data
    async fn migrate_service_data(&self, data: BTreeMap<ServiceId, Vec<u8>>) -> anyhow::Result<()> {
        for (service, sink) in self.service_data_sub.lock().await.iter() {
            if let Some(value) = data.get(service) {
                sink.send(SubscriptionMessage::from_json(value)?).await?;
            }
        }
        Ok(())
    }

    /// Migrate the service value
    async fn migrate_service_value(
        &self,
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

    /// Migrate the service preimage
    async fn migrate_service_preimage(
        &self,
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

    /// Migrate the service request
    async fn migrate_service_request(
        &self,
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
impl<C: runtime::Config> runtime::Hook for RpcHook<C> {
    // NOTE: since grandpa is not fully implemented, we set the best block
    // together with the finalized block.
    async fn on_finalized_block(
        &self,
        block: Block,
        diff: HashMap<OpaqueHash, Vec<u8>>,
    ) -> anyhow::Result<()> {
        let head = block.header.clone().try_into()?;
        self.runtime.storage.set_best(&head)?;
        self.runtime.storage.set_finalized(&head)?;

        // 1. dispatch the best and finalized block
        self.dispatch_best_block(&head.hash, head.slot as u64)
            .await?;
        self.dispatch_finalized_block(&head.hash, head.slot as u64)
            .await?;

        // 2. migrate states
        self.migrate_services(&head.hash, diff).await?;
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

impl<C: runtime::Config> Deref for RpcHook<C> {
    type Target = Rpc<C>;

    fn deref(&self) -> &Self::Target {
        &self.rpc
    }
}
