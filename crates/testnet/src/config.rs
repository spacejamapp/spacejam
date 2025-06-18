//! Test net configuration.

use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Deserialize)]
pub struct Testnet {
    /// The nodes in the testnet.
    pub node: BTreeMap<String, Node>,
}

/// A node in the testnet.
#[derive(Deserialize)]
pub struct Node {
    /// The binary name of the node.
    pub command: String,

    /// The path of the node specification.
    pub spec: PathBuf,

    /// The path of the node data.
    pub data: PathBuf,

    /// The QUIC address of the node.
    pub quic: String,

    /// The RPC address of the node.
    pub rpc: String,

    /// The extra arguments for the node.
    pub args: Vec<String>,

    /// The validator seed of the node.
    pub seed: String,
}

#[cfg(test)]
mod tests {
    use crate::Testnet;

    const CONFIG: &str = r#"
    [node.alice]
    command = "spacejam"
    spec = "alice.json"
    data = "alice"
    quic = "127.0.0.1:9944"
    rpc = "127.0.0.1:9933"
    args = []
    seed = "0"
    "#;

    #[test]
    fn parse_toml() {
        let testnet: Testnet = toml::from_str(CONFIG).unwrap();
        assert_eq!(testnet.node.len(), 1);
        assert_eq!(testnet.node.get("alice").unwrap().command, "spacejam");
    }
}
