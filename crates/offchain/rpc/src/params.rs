//! Parameters for the SpaceJam node.

use serde::{Deserialize, Serialize};

/// Parameters for spacejam
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Parameters {
    /// The parameters for version 1
    #[serde(rename = "V1")]
    pub v1: score::Parameters,
}
