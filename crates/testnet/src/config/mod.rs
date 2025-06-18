//! Test net configuration.

use serde::Deserialize;
use std::collections::BTreeMap;
pub use {network::Network, node::Node};

mod network;
mod node;

#[derive(Deserialize)]
pub struct Testnet {
    /// The nodes in the testnet.
    pub node: BTreeMap<String, Node>,

    /// The network configuration.
    pub network: Network,
}
