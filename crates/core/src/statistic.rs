use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents an activity record.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct ActivityRecord {
    pub blocks: u32,
    pub tickets: u32,
    pub pre_images: u32,
    pub pre_images_size: u32,
    pub guarantees: u32,
    pub assurances: u32,
}

/// Represents statistics.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct Statistics {
    #[json(nested)]
    pub current: Vec<ActivityRecord>,
    #[json(nested)]
    pub last: Vec<ActivityRecord>,
}
