use crate::{BeefyRoot, HeaderHash, OpaqueHash, StateRoot, TimeSlot};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents the RefineContext structure from ASN.1
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct RefineContext {
    #[json(hex)]
    pub anchor: HeaderHash,
    #[json(hex)]
    pub state_root: StateRoot,
    #[json(hex)]
    pub beefy_root: BeefyRoot,
    #[json(hex)]
    pub lookup_anchor: HeaderHash,
    pub lookup_anchor_slot: TimeSlot,
    #[json(hex)]
    pub prerequisites: Vec<OpaqueHash>,
}
