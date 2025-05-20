//! Chain Specifications

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Development chain spec
const DEV_SPEC: &str = include_str!("../../spec/dev/spec.json");

/// Chain Specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    /// List of bootnodes
    pub bootnodes: Vec<String>,

    /// The chain id
    pub id: String,

    /// genesis state kv map
    pub genesis_state: HashMap<String, String>,

    /// genesis header in hex format
    pub genesis_header: String,
}

impl Spec {
    /// Create a new dev chain spec
    pub fn dev() -> Self {
        serde_json::from_str(DEV_SPEC).expect("failed to parse dev spec")
    }
}
