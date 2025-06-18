//! Network configuration.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Network {
    /// The path of the network specification.
    pub spec: PathBuf,

    /// The log filters of the network.
    #[serde(default)]
    pub filter: Vec<String>,
}
