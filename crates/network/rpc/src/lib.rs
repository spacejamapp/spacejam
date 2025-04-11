//! Spacejam JSON RPC API.

use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::{core::SubscriptionResult, proc_macros::rpc};
use score::{CoreIndex, OpaqueHash, ServiceId, TimeSlot};
use serde::{Deserialize, Serialize};

/// Spacejam JSON RPC methods.
#[cfg_attr(all(feature = "client", feature = "server"), rpc(client, server))]
#[cfg_attr(feature = "server", rpc(server))]
#[cfg_attr(feature = "client", rpc(client))]
pub trait Api {
    /// Returns the header hash and slot of the head of the "best" chain.
    #[method(name = "bestBlock")]
    fn best_block(&self) -> Result<BlockResponse, ErrorObjectOwned>;

    /// Returns the header hash and slot of the latest finalized block.
    #[method(name = "finalizedBlock")]
    fn finalized_block(&self) -> Result<BlockResponse, ErrorObjectOwned>;

    /// Returns the header hash and slot of the parent of the block with the given
    /// header hash, or null if this is not known.
    #[method(name = "parent")]
    fn parent(&self, hash: OpaqueHash) -> Result<Option<BlockResponse>, ErrorObjectOwned>;

    /// Returns the state root of the block with the given header hash, or null if
    /// this is not known.
    #[method(name = "stateRoot")]
    fn state_root(&self, hash: OpaqueHash) -> Result<Option<OpaqueHash>, ErrorObjectOwned>;

    /// Returns the activity statistics stored in the posterior state of the block
    /// with the given header hash. The statistics are encoded as per the GP.
    ///
    /// null is returned if the block's posterior state is not known.
    #[method(name = "statistics")]
    fn statistics(&self, hash: OpaqueHash) -> Result<Option<Vec<u8>>, ErrorObjectOwned>;

    /// Returns the service info for the given service ID. The data are encoded as per the GP.
    ///
    /// null is returned if the block's posterior state is not known. Some(None) is returned if
    /// there is no value associated with the given service ID.
    #[method(name = "serviceInfo")]
    fn service_info(
        &self,
        hash: OpaqueHash,
        service: ServiceId,
    ) -> Result<Option<Vec<u8>>, ErrorObjectOwned>;

    /// Returns the value associated with the given service ID and key in the posterior state of
    /// the block with the given header hash. null is returned if there is no value associated with
    /// the given service ID and key.
    #[method(name = "serviceValue")]
    fn service_value(
        &self,
        hash: OpaqueHash,
        service: ServiceId,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, ErrorObjectOwned>;

    /// Returns the preimage for the given service ID and key in the posterior state of the block
    /// with the given header hash. null is returned if there is no preimage associated with the
    /// given service ID and key.
    #[method(name = "servicePreimage")]
    fn service_preimage(
        &self,
        hash: OpaqueHash,
        service: ServiceId,
        key: OpaqueHash,
    ) -> Result<Option<Vec<u8>>, ErrorObjectOwned>;

    /// Returns the preimage request associated with the given service ID and hash/len in the posterior
    /// state of the block with the given header hash. null is returned if there is no preimage request
    /// associated with the given service ID, hash and length.
    #[method(name = "serviceRequest")]
    fn service_request(&self, hash: OpaqueHash) -> Result<Option<Vec<u8>>, ErrorObjectOwned>;

    /// Returns the BEEFY root of the block with the given header hash, or null if this is not known.
    #[method(name = "beefyRoot")]
    fn beefy_root(&self, hash: OpaqueHash) -> Result<Option<Vec<u8>>, ErrorObjectOwned>;

    /// Submit a work-package to the guarantors currently assigned to the given core.
    #[method(name = "submitWorkPackage")]
    fn submit_work_package(
        &self,
        core: CoreIndex,
        package: Vec<u8>,
        extrinsics: Vec<Vec<u8>>,
    ) -> Result<(), ErrorObjectOwned>;

    /// Submit a preimage which is being requested by a given service.
    #[method(name = "submitPreimage")]
    fn submit_preimage(
        &self,
        service: ServiceId,
        preimage: Vec<u8>,
        hash: OpaqueHash,
    ) -> Result<(), ErrorObjectOwned>;

    /// Returns a list of all services currently known to be on JAM. This is a best-effort list and may
    /// not reflect the true state. Nodes could e.g. reasonably hide services which are not recently
    /// active from this list.
    #[method(name = "listServices")]
    fn list_services(&self, hash: OpaqueHash) -> Result<Vec<ServiceId>, ErrorObjectOwned>;

    /// Subscribe to updates of the head of the "best" chain, as returned by bestBlock.
    #[subscription(name = "subscribeBestBlock", item = BlockResponse)]
    fn subscribe_best_block(&self) -> Result<SubscriptionResult, ErrorObjectOwned>;

    /// Subscribe to updates of the latest finalized block, as returned by finalizedBlock.
    #[subscription(name = "subscribeFinalizedBlock", item = BlockResponse)]
    fn subscribe_finalized_block(&self) -> Result<SubscriptionResult, ErrorObjectOwned>;

    /// Subscribe to updates of the activity statistics stored in chain state. If finalized
    /// is true, the subscription will track the latest finalized block. If finalized is false,
    /// the subscription will track the head of the "best" chain. Note that in the latter case
    /// the reported statistics may never be included in the finalized chain.
    ///
    /// The statistics are encoded as per the GP.
    #[subscription(name = "subscribeStatistics", item = Vec<u8>)]
    fn subscribe_statistics(&self) -> Result<SubscriptionResult, ErrorObjectOwned>;

    /// Subscribe to updates of the service info for the given service ID. If finalized is true,
    /// the subscription will track the latest finalized block. If finalized is false, the subscription
    /// will track the head of the "best" chain. Note that in the latter case the reported service info
    /// may never be included in the finalized chain. The data are encoded as per the GP.
    #[subscription(name = "subscribeServiceInfo", item = Option<Vec<u8>>)]
    fn subscribe_service_info(&self) -> Result<SubscriptionResult, ErrorObjectOwned>;

    /// Subscribe to updates of the value associated with the given service ID and key. If finalized is true,
    /// the subscription will track the latest finalized block. If finalized is false, the subscription
    /// will track the head of the "best" chain. Note that in the latter case reported value changes
    /// may never be included in the finalized chain. The value field of subscription messages will be
    /// null when there is no value associated with the given service ID and key.
    #[subscription(name = "subscribeServiceValue", item = Option<Vec<u8>>)]
    fn subscribe_service_value(&self) -> Result<SubscriptionResult, ErrorObjectOwned>;

    /// Subscribe to updates of the preimage associated with the given service ID and hash. If finalized is true,
    /// the subscription will track the latest finalized block. If finalized is false, the subscription
    /// will track the head of the "best" chain. Note that in the latter case reported preimage changes
    /// may never be included in the finalized chain. The preimage field of subscription messages will be
    /// null when there is no preimage associated with the given service ID and hash.
    #[subscription(name = "subscribeServicePreimage", item = Option<Vec<u8>>)]
    fn subscribe_service_preimage(&self) -> Result<SubscriptionResult, ErrorObjectOwned>;

    /// Subscribe to updates of the preimage associated with the given service ID and hash. If finalized is true,
    /// the subscription will track the latest finalized block. If finalized is false, the subscription
    /// will track the head of the "best" chain. Note that in the latter case reported preimage changes
    /// may never be included in the finalized chain. The request field of subscription messages will be
    /// null when there is no preimage request associated with the given service ID, hash and length.
    #[subscription(name = "subscribeServiceRequest", item = Option<Vec<u8>>)]
    fn subscribe_service_request(&self) -> Result<SubscriptionResult, ErrorObjectOwned>;
}

/// Response for block info RPC call.
///
/// This is used in:
/// - `bestBlock`
/// - `subscribeBestBlock`
/// - `finalizedBlock`
/// - `subscribeFinalizedBlock`
/// - `parent`
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockResponse {
    /// The header hash
    pub hash: OpaqueHash,

    /// The slot
    pub slot: TimeSlot,
}
