//! Hooks for the Jadex runtime

use anyhow::Result;
use score::{block::Header, Block, OpaqueHash, ServiceId};
use std::{collections::HashMap, ops::Deref};

/// The jadex hook used in the spacejam runtime.
pub trait JadexHook {
    /// Called when a block is finalized
    fn on_finalized(&self, block: Header) -> Result<()>;

    /// Called when a service data is updated
    fn on_service_data(&self, service: ServiceId, data: HashMap<OpaqueHash, Vec<u8>>)
        -> Result<()>;

    /// Called when a service value is updated
    fn on_service_preimage(
        &self,
        service: ServiceId,
        data: HashMap<OpaqueHash, Vec<u8>>,
    ) -> Result<()>;

    /// Called when a service request is updated
    fn on_service_request(&self, data: Vec<(u32, Vec<u8>, Vec<u8>)>);

    /// Called when a service value is updated
    fn on_service_value(&self, service: ServiceId, data: HashMap<Vec<u8>, Vec<u8>>) -> Result<()>;
}

/// A hook that is used in the runtime
pub struct JadexHooked<T: JadexHook>(T);

impl<T: JadexHook> Deref for JadexHooked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: JadexHook + Send + Sync + 'static> runtime::Hook for JadexHooked<T> {
    async fn on_finalized_block(
        &self,
        block: Block,
        _diff: HashMap<OpaqueHash, Vec<u8>>,
    ) -> Result<()> {
        self.on_finalized(block.header)
    }
}
