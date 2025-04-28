//! The RPC server as the offchain component of SpaceJam

use anyhow::Result;
use async_trait::async_trait;
use rpc::{
    core::server::SubscriptionMessage, middleware, ApiServer, BlockResponse, ConnectionId,
    ErrorObjectOwned, PendingSubscriptionSink, RpcServiceBuilder, Server, SubscriptionResult,
    SubscriptionSink,
};
use runtime::{
    storage::{KVStorage, SyncStorage},
    Config, Runtime,
};
use score::{state::key, CoreIndex, OpaqueHash, ServiceId};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

/// Subscription handler
pub type Subscription = Arc<Mutex<HashMap<ConnectionId, SubscriptionSink>>>;

/// The RPC server for the offchain components of SpaceJam
pub struct Rpc<C: Config> {
    /// The runtime of the node
    pub runtime: Arc<Runtime<C>>,

    /// The best block subscription sinks
    pub best_block_sub: Subscription,

    /// The finalized block subscription sinks
    pub finalized_block_sub: Subscription,

    /// The statistics subscription sinks
    pub statistics_sub: Subscription,

    /// The service data subscription sinks
    pub service_data_sub: Subscription,

    /// The service value subscription sinks
    pub service_value_sub: Subscription,

    /// The service preimage subscription sinks
    pub service_preimage_sub: Subscription,

    /// The service request subscription sinks
    pub service_request_sub: Subscription,
}

impl<C: Config> Rpc<C> {
    /// Create a new RPC server
    pub fn new(runtime: Arc<Runtime<C>>) -> Self {
        Self {
            runtime,
            best_block_sub: Arc::new(Mutex::new(HashMap::new())),
            finalized_block_sub: Arc::new(Mutex::new(HashMap::new())),
            statistics_sub: Arc::new(Mutex::new(HashMap::new())),
            service_data_sub: Arc::new(Mutex::new(HashMap::new())),
            service_value_sub: Arc::new(Mutex::new(HashMap::new())),
            service_preimage_sub: Arc::new(Mutex::new(HashMap::new())),
            service_request_sub: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the JSON-RPC service
    pub async fn start(self, addr: SocketAddr) -> anyhow::Result<()> {
        let rpc_middleware = RpcServiceBuilder::new().layer_fn(middleware::Logger);
        let server = Server::builder()
            .set_rpc_middleware(rpc_middleware)
            .build(addr)
            .await?;

        // start the rpc server
        let addr = server.local_addr()?;
        tracing::info!("Listening RPC on {}", addr);
        server.start(self.into_rpc()).stopped().await;
        Ok(())
    }

    /// Dispatch the best block
    pub async fn dispatch_best_block(&self, hash: &OpaqueHash, slot: u64) -> Result<()> {
        for sink in self.best_block_sub.lock().await.values() {
            sink.send(SubscriptionMessage::from_json(&(hash, slot))?)
                .await?;
        }
        Ok(())
    }

    /// Dispatch the finalized block
    pub async fn dispatch_finalized_block(&self, hash: &OpaqueHash, slot: u64) -> Result<()> {
        for sink in self.finalized_block_sub.lock().await.values() {
            sink.send(SubscriptionMessage::from_json(&(hash, slot))?)
                .await?;
        }
        Ok(())
    }

    /// Dispatch the statistics
    pub async fn dispatch_statistics(&self, blob: &[u8]) -> Result<()> {
        for sink in self.statistics_sub.lock().await.values() {
            sink.send(SubscriptionMessage::from_json(&blob)?).await?;
        }
        Ok(())
    }

    /// Clone the RPC server
    pub fn cloned(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            best_block_sub: self.best_block_sub.clone(),
            finalized_block_sub: self.finalized_block_sub.clone(),
            statistics_sub: self.statistics_sub.clone(),
            service_data_sub: self.service_data_sub.clone(),
            service_value_sub: self.service_value_sub.clone(),
            service_preimage_sub: self.service_preimage_sub.clone(),
            service_request_sub: self.service_request_sub.clone(),
        }
    }
}

#[async_trait]
impl<C: Config> ApiServer for Rpc<C> {
    fn best_block(&self) -> Result<BlockResponse, ErrorObjectOwned> {
        let best = self
            .runtime
            .storage
            .get_best()
            .map_err(|e| {
                ErrorObjectOwned::owned(
                    1,
                    format!("Best head not found, {e:?}"),
                    Option::<()>::None,
                )
            })
            .unwrap_or_default();
        Ok((best.hash, best.slot))
    }

    fn finalized_block(&self) -> Result<BlockResponse, ErrorObjectOwned> {
        let finalized = self
            .runtime
            .storage
            .get_finalized()
            .map_err(|e| {
                ErrorObjectOwned::owned(
                    1,
                    format!("Finalized head not found, {e:?}"),
                    Option::<()>::None,
                )
            })
            .unwrap_or_default();
        Ok((finalized.hash, finalized.slot))
    }

    fn parent(&self, hash: OpaqueHash) -> Result<Option<BlockResponse>, ErrorObjectOwned> {
        let parent = self.runtime.storage.get_parent(&hash).map_err(|e| {
            ErrorObjectOwned::owned(1, format!("Parent not found: {e:?}"), Option::<()>::None)
        })?;
        Ok(Some((parent.hash, parent.slot)))
    }

    fn state_root(&self, hash: OpaqueHash) -> Result<Option<OpaqueHash>, ErrorObjectOwned> {
        let state_root = self.runtime.storage.get_state_root(&hash).map_err(|e| {
            ErrorObjectOwned::owned(
                1,
                format!("State root not found: {e:?}"),
                Option::<()>::None,
            )
        })?;
        Ok(Some(state_root))
    }

    fn statistics(&self, hash: OpaqueHash) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        let key = [hash.as_ref(), key::STATISTICS.as_ref()].concat();
        let statistics = self.runtime.storage.get(&key).map_err(|e| {
            ErrorObjectOwned::owned(
                1,
                format!("Statistics not found: {e:?}"),
                Option::<()>::None,
            )
        })?;
        Ok(statistics)
    }

    // TODO: need to do snapshot for block state
    fn service_data(
        &self,
        _hash: OpaqueHash,
        _service: ServiceId,
    ) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to do snapshot for block state",
            Option::<()>::None,
        ))
    }

    // TODO: need to do snapshot for block state
    fn service_value(
        &self,
        _hash: OpaqueHash,
        _service: ServiceId,
        _key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to do snapshot for block state",
            Option::<()>::None,
        ))
    }

    // TODO: need to do snapshot for block state
    fn service_preimage(
        &self,
        _hash: OpaqueHash,
        _service: ServiceId,
        _key: OpaqueHash,
    ) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to do snapshot for block state",
            Option::<()>::None,
        ))
    }

    // TODO: need to do snapshot for block state
    fn service_request(&self, _hash: OpaqueHash) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to do snapshot for block state",
            Option::<()>::None,
        ))
    }

    // TODO: need to do snapshot for block state
    fn beefy_root(&self, hash: OpaqueHash) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        let key = [hash.as_ref(), b"beefy_root"].concat();
        let beefy_root = self.runtime.storage.get(&key).map_err(|e| {
            ErrorObjectOwned::owned(
                1,
                format!("Beefy root not found: {e:?}"),
                Option::<()>::None,
            )
        })?;
        Ok(beefy_root)
    }

    // TODO: need to figure out the usage of core and package
    fn submit_work_package(
        &self,
        _core: CoreIndex,
        _package: Vec<u8>,
        _extrinsics: Vec<Vec<u8>>,
    ) -> Result<(), ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to do snapshot for block state",
            Option::<()>::None,
        ))
    }

    // TODO: need to do snapshot for block state
    fn submit_preimage(
        &self,
        _service: ServiceId,
        _preimage: Vec<u8>,
        _hash: OpaqueHash,
    ) -> Result<(), ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to do snapshot for block state",
            Option::<()>::None,
        ))
    }

    fn list_services(&self, hash: OpaqueHash) -> Result<Vec<ServiceId>, ErrorObjectOwned> {
        let key = [hash.as_ref(), b"services"].concat();
        let services = self.runtime.storage.get(&key).map_err(|e| {
            ErrorObjectOwned::owned(1, format!("Services not found: {e:?}"), Option::<()>::None)
        })?;

        let Some(services) = services else {
            return Ok(vec![]);
        };

        codec::decode(&services).map_err(|e| {
            ErrorObjectOwned::owned(
                1,
                format!("Failed to decode services: {e:?}"),
                Option::<()>::None,
            )
        })
    }

    async fn subscribe_best_block(&self, sink: PendingSubscriptionSink) -> SubscriptionResult {
        let accepted = sink.accept().await?;

        self.best_block_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }

    async fn subscribe_finalized_block(&self, sink: PendingSubscriptionSink) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.finalized_block_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }

    async fn subscribe_statistics(
        &self,
        sink: PendingSubscriptionSink,
        _finalized: bool,
    ) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.statistics_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }

    async fn subscribe_service_data(
        &self,
        sink: PendingSubscriptionSink,
        service: ServiceId,
        _finalized: bool,
    ) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.service_data_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }

    async fn subscribe_service_value(
        &self,
        sink: PendingSubscriptionSink,
        service: ServiceId,
        key: Vec<u8>,
        _finalized: bool,
    ) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.service_value_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }

    async fn subscribe_service_preimage(
        &self,
        sink: PendingSubscriptionSink,
        service: ServiceId,
        hash: OpaqueHash,
        _finalized: bool,
    ) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.service_preimage_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }

    async fn subscribe_service_request(
        &self,
        sink: PendingSubscriptionSink,
        service: ServiceId,
        hash: OpaqueHash,
        length: u32,
        _finalized: bool,
    ) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.service_request_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }
}
