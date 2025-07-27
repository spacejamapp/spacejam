//! Refine types

use crate::{BeefyRoot, OpaqueHash, StateRoot, TimeSlot};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents the RefineContext structure from ASN.1
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct RefineContext {
    /// The anchor
    #[json(hex)]
    pub anchor: OpaqueHash,

    /// The state root
    #[json(hex)]
    pub state_root: StateRoot,

    /// The beefy root
    #[json(hex)]
    pub beefy_root: BeefyRoot,

    /// The lookup anchor
    #[json(hex)]
    pub lookup_anchor: OpaqueHash,

    /// The lookup anchor slot
    pub lookup_anchor_slot: TimeSlot,

    /// The prerequisites
    #[json(hex)]
    pub prerequisites: Vec<OpaqueHash>,
}

/// The refine load
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct RefineLoad {
    /// The gas used
    #[serde(with = "codec::compact")]
    pub gas_used: u64,

    /// The number of imports
    #[serde(with = "codec::compact")]
    pub imports: u16,

    /// The number of extrinsics
    #[serde(with = "codec::compact")]
    pub extrinsic_count: u16,

    /// The size of the extrinsics
    #[serde(with = "codec::compact")]
    pub extrinsic_size: u32,

    /// The number of exports
    #[serde(with = "codec::compact")]
    pub exports: u16,
}
