//! Auditor abstraction

use crate::{d3l::Justification, DataLake};
use anyhow::Result;

/// Auditor abstraction
#[allow(async_fn_in_trait)]
pub trait Auditor: DataLake {
    /// Audit a shard (CE138)
    async fn audit(&self, _shard: &[u8], _justification: &Justification) -> Result<()> {
        todo!()
    }
}
