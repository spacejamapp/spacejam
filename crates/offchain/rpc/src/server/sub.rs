//! Subscription handlers for the Spacejam JSON RPC API.

use anyhow::Result;
use jsonrpsee::{SubscriptionMessage, SubscriptionSink};
use score::{OpaqueHash, ServiceId};
use std::sync::Arc;
use tokio::sync::Mutex;

/// The subscription type
pub type SubscriptionFilter<T> = Arc<Mutex<Vec<(T, SubscriptionSink)>>>;

/// The raw subscription type
pub type Subscription = Arc<Mutex<Vec<SubscriptionSink>>>;

/// Subscription manager
#[derive(Default, Clone)]
pub struct SubscriptionManager {
    /// The best block subscription sinks
    pub best_block_sub: Subscription,

    /// The finalized block subscription sinks
    pub finalized_block_sub: Subscription,

    /// The statistics subscription sinks
    pub statistics_sub: Subscription,

    /// The service data subscription sinks
    pub service_data_sub: SubscriptionFilter<ServiceId>,

    /// The service value subscription sinks
    pub service_value_sub: SubscriptionFilter<ServiceValueFilter>,

    /// The service preimage subscription sinks
    pub service_preimage_sub: SubscriptionFilter<ServicePreimageFilter>,

    /// The service request subscription sinks
    pub service_request_sub: SubscriptionFilter<ServiceRequestFilter>,
}

impl SubscriptionManager {
    /// Dispatch the best block
    pub async fn dispatch_best_block(&self, hash: &OpaqueHash, slot: u64) -> Result<()> {
        for sink in self.best_block_sub.lock().await.iter() {
            sink.send(SubscriptionMessage::from_json(&(hash, slot))?)
                .await?;
        }
        Ok(())
    }

    /// Dispatch the finalized block
    pub async fn dispatch_finalized_block(&self, hash: &OpaqueHash, slot: u64) -> Result<()> {
        for sink in self.finalized_block_sub.lock().await.iter() {
            sink.send(SubscriptionMessage::from_json(&(hash, slot))?)
                .await?;
        }
        Ok(())
    }

    /// Dispatch the statistics
    pub async fn dispatch_statistics(&self, blob: &[u8]) -> Result<()> {
        for sink in self.statistics_sub.lock().await.iter() {
            sink.send(SubscriptionMessage::from_json(&blob)?).await?;
        }
        Ok(())
    }
}

/// The service value filter
pub struct ServiceValueFilter {
    /// The service ID
    pub service: ServiceId,

    /// The key
    pub key: Vec<u8>,
}

impl From<(ServiceId, Vec<u8>)> for ServiceValueFilter {
    fn from((service, key): (ServiceId, Vec<u8>)) -> Self {
        Self { service, key }
    }
}

/// The service preimage filter
pub struct ServicePreimageFilter {
    /// The service ID
    pub service: ServiceId,

    /// The hash
    pub hash: OpaqueHash,
}

impl From<(ServiceId, OpaqueHash)> for ServicePreimageFilter {
    fn from((service, hash): (ServiceId, OpaqueHash)) -> Self {
        Self { service, hash }
    }
}

/// The service request filter
pub struct ServiceRequestFilter {
    /// The service ID
    pub service: ServiceId,

    /// The hash
    pub hash: OpaqueHash,

    /// The length
    pub length: u32,
}

impl From<(ServiceId, OpaqueHash, u32)> for ServiceRequestFilter {
    fn from((service, hash, length): (ServiceId, OpaqueHash, u32)) -> Self {
        Self {
            service,
            hash,
            length,
        }
    }
}
