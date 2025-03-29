//! Accumulation of guarantees

use crate::{service::WorkReport, OpaqueHash, TimeSlot};

/// (b) Accumulate the available work reports
pub fn accumulate(_slot: TimeSlot, _reports: Vec<WorkReport>) -> anyhow::Result<OpaqueHash> {
    Ok(Default::default())
}
