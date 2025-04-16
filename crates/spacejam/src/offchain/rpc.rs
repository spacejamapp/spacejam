//! The RPC server as the offchain component of SpaceJam

use anyhow::Result;
use async_trait::async_trait;
use rpc::{
    ApiServer, BlockResponse, ConnectionId, ErrorObjectOwned, PendingSubscriptionSink,
    SubscriptionResult, SubscriptionSink,
};
use runtime::{storage::SyncStorage, Config, Runtime};
use score::{CoreIndex, OpaqueHash, ServiceId};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// The RPC server for the offchain components of SpaceJam
pub struct Rpc<C: Config> {
    /// The runtime of the node
    pub runtime: Arc<Runtime<C>>,

    /// The best block subscription sinks
    pub best_block_sub: Mutex<HashMap<ConnectionId, SubscriptionSink>>,

    /// The finalized block subscription sinks
    pub finalized_block_sub: Mutex<HashMap<ConnectionId, SubscriptionSink>>,

    /// The statistics subscription sinks
    pub statistics_sub: Mutex<HashMap<ConnectionId, SubscriptionSink>>,

    /// The service info subscription sinks
    pub service_info_sub: Mutex<HashMap<ConnectionId, SubscriptionSink>>,

    /// The service value subscription sinks
    pub service_value_sub: Mutex<HashMap<ConnectionId, SubscriptionSink>>,

    /// The service preimage subscription sinks
    pub service_preimage_sub: Mutex<HashMap<ConnectionId, SubscriptionSink>>,

    /// The service request subscription sinks
    pub service_request_sub: Mutex<HashMap<ConnectionId, SubscriptionSink>>,
}

impl<C: Config> Rpc<C> {
    /// Create a new RPC server
    pub fn new(runtime: Arc<Runtime<C>>) -> Self {
        Self {
            runtime,
            best_block_sub: Mutex::new(HashMap::new()),
            finalized_block_sub: Mutex::new(HashMap::new()),
            statistics_sub: Mutex::new(HashMap::new()),
            service_info_sub: Mutex::new(HashMap::new()),
            service_value_sub: Mutex::new(HashMap::new()),
            service_preimage_sub: Mutex::new(HashMap::new()),
            service_request_sub: Mutex::new(HashMap::new()),
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
            .map_err(|_| ErrorObjectOwned::owned(1, "Best head not found", Option::<()>::None))
            .unwrap_or_default();
        Ok((best.hash, best.slot))
    }

    fn finalized_block(&self) -> Result<BlockResponse, ErrorObjectOwned> {
        let finalized = self
            .runtime
            .storage
            .get_finalized()
            .map_err(|_| ErrorObjectOwned::owned(1, "Finalized head not found", Option::<()>::None))
            .unwrap_or_default();
        Ok((finalized.hash, finalized.slot))
    }

    // TODO: store the parent in the storage
    fn parent(&self, _hash: OpaqueHash) -> Result<Option<BlockResponse>, ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to store the parent in the storage instead of memory",
            Option::<()>::None,
        ))
    }

    // TODO: need to do snapshot for block state
    fn state_root(&self, _hash: OpaqueHash) -> Result<Option<OpaqueHash>, ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to do snapshot for block state",
            Option::<()>::None,
        ))
    }

    // TODO: need to do snapshot for block state
    fn statistics(&self, _hash: OpaqueHash) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to do snapshot for block state",
            Option::<()>::None,
        ))
    }

    // TODO: need to do snapshot for block state
    fn service_info(
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
    fn beefy_root(&self, _hash: OpaqueHash) -> Result<Option<Vec<u8>>, ErrorObjectOwned> {
        Err(ErrorObjectOwned::owned(
            1,
            "Not yet implemented, need to do snapshot for block state",
            Option::<()>::None,
        ))
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

    // TODO: need to do snapshot for block state
    fn list_services(&self, _hash: OpaqueHash) -> Result<Vec<ServiceId>, ErrorObjectOwned> {
        Ok(vec![])
        // Err(ErrorObjectOwned::owned(
        //     1,
        //     "Not yet implemented, need to do snapshot for block state",
        //     Option::<()>::None,
        // ))
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

    async fn subscribe_statistics(&self, sink: PendingSubscriptionSink) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.statistics_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }

    async fn subscribe_service_info(&self, sink: PendingSubscriptionSink) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.service_info_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }

    async fn subscribe_service_value(&self, sink: PendingSubscriptionSink) -> SubscriptionResult {
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
    ) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.service_preimage_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }

    async fn subscribe_service_request(&self, sink: PendingSubscriptionSink) -> SubscriptionResult {
        let accepted = sink.accept().await?;
        self.service_request_sub
            .lock()
            .await
            .insert(accepted.connection_id(), accepted);
        Ok(())
    }
}
