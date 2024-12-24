use crate::ServiceId;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a preimage request.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct Preimage {
    pub requester: ServiceId,
    #[json(hex)]
    pub blob: Vec<u8>,
}

/// Represents a sequence of preimages.
pub type PreimagesExtrinsic = Vec<Preimage>;
