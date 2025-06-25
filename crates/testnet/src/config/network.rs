//! Network configuration.

use crate::config::Filter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Network {
    /// The path of the network specification.
    pub spec: PathBuf,

    /// The global log filters of the network.
    #[serde(default, flatten)]
    pub filter: Filter,

    /// The watch list of the nodes in the network.
    #[serde(default)]
    pub watch: Vec<String>,
}
