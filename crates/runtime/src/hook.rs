//! Hooks for the runtime

use crate::Head;
use anyhow::Result;
use score::{
    OpaqueHash, ServiceId,
    service::{ServiceAccountData, WorkReport},
    statistic::Statistics,
};

/// Hooks for the runtime
pub trait Hook {
    /// Called when a new best block is imported
    fn on_best_block(&self, _block: Head) -> Result<()> {
        Ok(())
    }

    /// Called when a new finalized block is imported
    fn on_finalized_block(&self, _block: Head) -> Result<()> {
        Ok(())
    }

    /// Called when statistics get updated
    fn on_statistics(&self, _stats: Statistics) -> Result<()> {
        Ok(())
    }

    /// Called when service info get updated
    fn on_service_info(&self, _id: ServiceId, _info: ServiceAccountData) -> Result<()> {
        Ok(())
    }

    /// Called when a service value get updated
    fn on_service_value(&self, _id: ServiceId, _key: Vec<u8>, _value: Vec<u8>) -> Result<()> {
        Ok(())
    }

    /// Called when a service preimage get updated
    fn on_service_preimage(&self, _id: ServiceId, _hash: OpaqueHash, _blob: Vec<u8>) -> Result<()> {
        Ok(())
    }

    /// Called when a new service request finalized
    fn on_service_request(&self, _id: ServiceId, _request: WorkReport) -> Result<()> {
        Ok(())
    }
}

impl Hook for () {}
