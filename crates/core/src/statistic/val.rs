//! Validator activities

use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents an activity record.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Copy, Default)]
pub struct ValidatorActivityRecord {
    /// (b) Number of blocks produced
    pub blocks: u32,

    /// (t) Number of tickets
    pub tickets: u32,

    /// (p) Number of pre-images
    pub pre_images: u32,

    /// (d) Size of pre-images
    pub pre_images_size: u32,

    /// (g) Number of guarantees
    pub guarantees: u32,

    /// (a) Number of assurances
    pub assurances: u32,
}
