//! Service statistics

use crate::Gas;
use codec::Compact;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a service record.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct ServiceActivityRecord {
    /// Number of preimages provided to this service.
    #[json(compact)]
    pub provided_count: Compact<u16>,
    /// Total size of preimages provided to this service.
    #[json(compact)]
    pub provided_size: Compact<u32>,
    /// Number of work-items refined by service for reported work.
    #[json(compact)]
    pub refinement_count: Compact<u32>,
    /// Amount of gas used for refinement by service for reported work.
    #[json(compact)]
    pub refinement_gas_used: Compact<Gas>,
    /// Number of segments imported from the DL by service for reported work.
    #[json(compact)]
    pub imports: Compact<u32>,
    /// Number of segments exported into the DL by service for reported work.
    #[json(compact)]
    pub exports: Compact<u32>,
    /// Total size of extrinsics used by service for reported work.
    #[json(compact)]
    pub extrinsic_size: Compact<u32>,
    /// Total number of extrinsics used by service for reported work.
    #[json(compact)]
    pub extrinsic_count: Compact<u32>,
    /// Number of work-items accumulated by service.
    #[json(compact)]
    pub accumulate_count: Compact<u32>,
    /// Amount of gas used for accumulation by service.
    #[json(compact)]
    pub accumulate_gas_used: Compact<Gas>,
    /// Number of transfers processed by service.
    #[json(compact)]
    pub on_transfers_count: Compact<u32>,
    /// Amount of gas used for processing transfers by service.
    #[json(compact)]
    pub on_transfers_gas_used: Compact<Gas>,
}
