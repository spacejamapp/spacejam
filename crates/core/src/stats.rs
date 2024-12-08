use codec::Json;
use serde::{Deserialize, Serialize};

/// Represents an activity record.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ActivityRecord {
    blocks: u32,
    tickets: u32,
    pre_images: u32,
    pre_images_size: u32,
    guarantees: u32,
    assurances: u32,
}

/// Represents statistics.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct Statistics {
    #[json(nested)]
    current: Vec<ActivityRecord>,
    #[json(nested)]
    last: Vec<ActivityRecord>,
}
