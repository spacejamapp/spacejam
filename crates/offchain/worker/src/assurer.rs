//! Assurer abstraction

use crate::{d3l::Justification, DataLake};
use anyhow::Result;
use score::{service::WorkReport, OpaqueHash, Segment};

/// Assurer abstraction
#[allow(async_fn_in_trait)]
pub trait Assurer: DataLake {
    /// On receiving a request of a work report (CE136)
    async fn work_report(&self, _hash: OpaqueHash) -> Result<WorkReport> {
        todo!()
    }

    /// On receiving a request of a audit shard (CE137)
    async fn audit_shard(
        &self,
        _erasure_root: OpaqueHash,
        _shard_index: u16,
    ) -> Result<(Vec<u8>, Justification)> {
        todo!()
    }

    /// On receiving a request of a segment (CE139/CE140)
    async fn segment(
        &self,
        _erasure_root: OpaqueHash,
        _shard_index: u16,
        _indices: Vec<u16>,
    ) -> Result<Vec<(Segment, Justification)>> {
        todo!()
    }
}
