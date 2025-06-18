//! Network configuration.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
pub struct Network {
    /// The path of the network specification.
    pub spec: PathBuf,

    /// The global log filters of the network.
    #[serde(default)]
    pub filter: Vec<String>,

    /// The watch list of the nodes in the network.
    #[serde(default)]
    pub watch: Vec<String>,
}
