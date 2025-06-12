//! Hooks for the runtime

use anyhow::Result;
use score::{Block, OpaqueHash, ServiceId, StorageKey, block::Head, state::key};
use std::collections::{BTreeMap, HashMap};

use crate::storage::Commit;

/// Hooks for the runtime
pub trait Hook: Send + Sync {
    /// Called when a new best block is imported
    fn on_best_block(&self, _block: Head) -> Result<()> {
        Ok(())
    }

    /// Called when a new finalized block is imported
    fn on_finalized_block(&self, _block: Block) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Called when a key-value pair is updated
    fn on_key_value(&self, _hash: OpaqueHash, _key: StorageKey, _value: &[u8]) -> Result<()> {
        Ok(())
    }

    /// Called when a diff is applied
    fn on_diff(
        &self,
        hash: OpaqueHash,
        diff: Commit<StorageKey, Vec<u8>>,
    ) -> impl Future<Output = Result<()>> + Send {
        async move {
            let mut data = BTreeMap::new();
            let mut preimage = BTreeMap::new();
            let mut request = BTreeMap::new();
            let mut svalue = BTreeMap::new();

            // FIXME: support removal
            for (key, value) in diff.iset() {
                // skip the key that is not related to service
                if key[1..].iter().all(|b| *b == 0) {
                    continue;
                }

                // call the hook
                self.on_key_value(hash, key, &value)?;

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

            tokio::try_join!(
                self.on_service_data(hash, data),
                self.on_service_value(hash, svalue),
                self.on_service_preimage(hash, preimage),
                self.on_service_request(hash, request)
            )?;

            Ok(())
        }
    }

    /// Called when a service data is updated
    fn on_service_data(
        &self,
        _hash: OpaqueHash,
        _data: BTreeMap<ServiceId, Vec<u8>>,
    ) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Called when a service value is updated
    fn on_service_value(
        &self,
        _hash: OpaqueHash,
        _data: BTreeMap<ServiceId, (Vec<u8>, Vec<u8>)>,
    ) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Called when a service preimage is updated
    fn on_service_preimage(
        &self,
        _hash: OpaqueHash,
        _data: BTreeMap<ServiceId, (Vec<u8>, Vec<u8>)>,
    ) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Called when a service request is updated
    fn on_service_request(
        &self,
        _hash: OpaqueHash,
        _data: BTreeMap<ServiceId, (u32, Vec<u8>, Vec<u8>)>,
    ) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}

impl Hook for () {}
