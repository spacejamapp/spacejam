//! The RPC server as the offchain component of SpaceJam

use anyhow::Result;
use async_trait::async_trait;
use rpc::{
    server::middleware, server::SubscriptionManager, ApiServer, BlockResponse, ErrorObjectOwned,
    PendingSubscriptionSink, RpcServiceBuilder, Server, SubscriptionResult,
};
use runtime::{
    storage::{KVStorage, SyncStorage},
    Config, Runtime,
};
use score::{
    state::{account, key},
    CoreIndex, OpaqueHash, ServiceId,
};
use std::{net::SocketAddr, ops::Deref, sync::Arc};

/// The RPC server for the offchain components of SpaceJam
pub struct Rpc<C: Config> {
    /// The runtime of the node
    pub runtime: Arc<Runtime<C>>,

    /// The subscription manager
    pub manager: SubscriptionManager,
}

impl<C: Config> Rpc<C> {
    /// Create a new RPC server
    pub fn new(runtime: Arc<Runtime<C>>) -> Self {
        Self {
            runtime,
            manager: SubscriptionManager::default(),
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
}

#[async_trait]
impl<C: Config> ApiServer for Rpc<C> {
    fn best_block(&self) -> Result<BlockResponse, ErrorObjectOwned> {
        let best = self.runtime.storage.best().map_err(to_owned_error)?;
        Ok((best.hash, best.slot))
    }

    fn finalized_block(&self) -> Result<BlockResponse, ErrorObjectOwned> {
        let finalized = self.runtime.storage.finalized().map_err(to_owned_error)?;
        Ok((finalized.hash, finalized.slot))
    }

    fn parent(&self, hash: OpaqueHash) -> Result<Option<BlockResponse>, ErrorObjectOwned> {
        let parent = self.runtime.storage.parent(&hash).map_err(to_owned_error)?;
        let header = self.runtime.storage.header(&hash).map_err(to_owned_error)?;
        Ok(parent.map(|parent| (parent, header.slot)))
    }

    fn state_root(&self, hash: OpaqueHash) -> Result<Option<OpaqueHash>, ErrorObjectOwned> {
        let state_root = self
            .runtime
            .storage
            .state_root(&hash)
            .map_err(to_owned_error)?;
        Ok(state_root)
    }

    fn statistics(&self, hash: OpaqueHash) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        let key = [hash.as_ref(), key::STATISTICS.as_ref()].concat();
        let statistics = self.runtime.storage.get(&key).map_err(to_owned_error)?;
        Ok(statistics)
    }

    fn service_data(
        &self,
        hash: OpaqueHash,
        service: ServiceId,
    ) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        let info = account::info(service);
        let key = [hash.as_ref(), info.as_ref()].concat();
        let data = self.runtime.storage.get(&key).map_err(to_owned_error)?;
        Ok(data)
    }

    fn service_value(
        &self,
        hash: OpaqueHash,
        service: ServiceId,
        key: OpaqueHash,
    ) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        let value = account::storage(service, &key);
        let key = [hash.as_ref(), value.as_ref()].concat();
        let data = self.runtime.storage.get(&key).map_err(to_owned_error)?;
        Ok(data)
    }

    fn service_preimage(
        &self,
        hash: OpaqueHash,
        service: ServiceId,
        key: OpaqueHash,
    ) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        let pkey = account::preimage(service, key);
        let key = [hash.as_ref(), pkey.as_ref()].concat();
        let data = self.runtime.storage.get(&key).map_err(to_owned_error)?;
        Ok(data)
    }

    fn service_request(
        &self,
        header_hash: OpaqueHash,
        service: ServiceId,
        hash: OpaqueHash,
        length: u32,
    ) -> Result<Option<Vec<u32>>, ErrorObjectOwned> {
        let lkey = account::lookup(service, length, hash);
        let key = [header_hash.as_ref(), lkey.as_ref()].concat();
        let data = self.runtime.storage.get(&key).map_err(to_owned_error)?;

        let Some(data) = data else {
            return Ok(None);
        };

        let data = codec::decode(&data).map_err(to_owned_error)?;

        Ok(Some(data))
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

    fn submit_work_package(
        &self,
        _core: CoreIndex,
        _package: Vec<u8>,
        _extrinsics: Vec<Vec<u8>>,
    ) -> Result<(), ErrorObjectOwned> {
        Err(to_owned_error(
            "Not yet implemented, need to do snapshot for block state",
        ))
    }

    fn submit_preimage(
        &self,
        _service: ServiceId,
        _preimage: Vec<u8>,
        _hash: OpaqueHash,
    ) -> Result<(), ErrorObjectOwned> {
        Err(to_owned_error(
            "Not yet implemented, need to do snapshot for block state",
        ))
    }

    fn list_services(&self, hash: OpaqueHash) -> Result<Vec<ServiceId>, ErrorObjectOwned> {
        let key = [hash.as_ref(), b"services"].concat();
        let services = self.runtime.storage.get(&key).map_err(to_owned_error)?;
        let Some(services) = services else {
            return Ok(vec![]);
        };

        let services = codec::decode(&services).map_err(to_owned_error)?;
        Ok(services)
    }

    async fn subscribe_best_block(&self, sink: PendingSubscriptionSink) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.best_block_sub.lock().await.push(accepted);
        Ok(())
    }

    async fn subscribe_finalized_block(&self, sink: PendingSubscriptionSink) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.finalized_block_sub.lock().await.push(accepted);
        Ok(())
    }

    async fn subscribe_statistics(
        &self,
        sink: PendingSubscriptionSink,
        _finalized: bool,
    ) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.statistics_sub.lock().await.push(accepted);
        Ok(())
    }

    async fn subscribe_service_data(
        &self,
        sink: PendingSubscriptionSink,
        service: ServiceId,
        _finalized: bool,
    ) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.service_data_sub.lock().await.push((service, accepted));
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
            .push(((service, key).into(), accepted));
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
            .push(((service, hash).into(), accepted));
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
            .push(((service, hash, length).into(), accepted));
        Ok(())
    }
}

impl<C: Config> Deref for Rpc<C> {
    type Target = SubscriptionManager;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

impl<C: Config> Clone for Rpc<C> {
    fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            manager: self.manager.clone(),
        }
    }
}

/// Convert the error to an owned error
fn to_owned_error(e: impl ToString) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(1, e.to_string(), Option::<()>::None)
}
