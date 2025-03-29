//! Refine types

use crate::{BeefyRoot, HeaderHash, OpaqueHash, StateRoot, TimeSlot};
use codec::Compact;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents the RefineContext structure from ASN.1
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct RefineContext {
    /// The anchor
    #[json(hex)]
    pub anchor: HeaderHash,

    /// The state root
    #[json(hex)]
    pub state_root: StateRoot,

    /// The beefy root
    #[json(hex)]
    pub beefy_root: BeefyRoot,

    /// The lookup anchor
    #[json(hex)]
    pub lookup_anchor: HeaderHash,

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
    #[json(compact)]
    pub gas_used: Compact<u64>,

    /// The number of imports
    #[json(compact)]
    pub imports: Compact<u16>,

    /// The number of extrinsics
    #[json(compact)]
    pub extrinsic_count: Compact<u16>,

    /// The size of the extrinsics
    #[json(compact)]
    pub extrinsic_size: Compact<u32>,

    /// The number of exports
    #[json(compact)]
    pub exports: Compact<u16>,
}
