//! Service statistics

use crate::Gas;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a service record (13.7).
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct ServiceActivityRecord {
    /// Number of preimages provided to this service.
    #[serde(with = "codec::compact")]
    pub provided_count: u16,
    /// Total size of preimages provided to this service.
    #[serde(with = "codec::compact")]
    pub provided_size: u32,
    /// Number of work-items refined by service for reported work.
    #[serde(with = "codec::compact")]
    pub refinement_count: u32,
    /// Amount of gas used for refinement by service for reported work.
    #[serde(with = "codec::compact")]
    pub refinement_gas_used: Gas,
    /// Number of segments imported from the DL by service for reported work.
    #[serde(with = "codec::compact")]
    pub imports: u32,
    /// Total number of extrinsics used by service for reported work.
    #[serde(with = "codec::compact")]
    pub extrinsic_count: u32,
    /// Total size of extrinsics used by service for reported work.
    #[serde(with = "codec::compact")]
    pub extrinsic_size: u32,
    /// Number of segments exported into the DL by service for reported work.
    #[serde(with = "codec::compact")]
    pub exports: u32,
    /// Number of work-items accumulated by service.
    #[serde(with = "codec::compact")]
    pub accumulate_count: u32,
    /// Amount of gas used for accumulation by service.
    #[serde(with = "codec::compact")]
    pub accumulate_gas_used: Gas,
}
