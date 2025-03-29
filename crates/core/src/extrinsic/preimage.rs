//! Preimage extrinsic

use crate::ServiceId;
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::cmp::Ordering;

/// Represents a sequence of preimages.
pub type PreimagesExtrinsic = Vec<Preimage>;

/// Represents a preimage request.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Hash)]
pub struct Preimage {
    pub requester: ServiceId,
    #[json(hex)]
    pub blob: Vec<u8>,
}

impl PartialOrd for Preimage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.requester == other.requester {
            Some(self.blob.cmp(&other.blob))
        } else {
            Some(self.requester.cmp(&other.requester))
        }
    }
}
