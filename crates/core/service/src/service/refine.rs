//! Refine types

use crate::{BeefyRoot, OpaqueHash, StateRoot, TimeSlot, Vec};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use {crate::String, spacejson::Json};

/// Represents the RefineContext structure from ASN.1
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct RefineContext {
    /// The anchor
    #[cfg_attr(feature = "json", json(hex))]
    pub anchor: OpaqueHash,

    /// The state root
    #[cfg_attr(feature = "json", json(hex))]
    pub state_root: StateRoot,

    /// The beefy root
    #[cfg_attr(feature = "json", json(hex))]
    pub beefy_root: BeefyRoot,

    /// The lookup anchor
    #[cfg_attr(feature = "json", json(hex))]
    pub lookup_anchor: OpaqueHash,

    /// The lookup anchor slot
    pub lookup_anchor_slot: TimeSlot,

    /// The prerequisites
    #[cfg_attr(feature = "json", json(hex))]
    pub prerequisites: Vec<OpaqueHash>,
}

/// The refine load
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "json", derive(Json))]
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
