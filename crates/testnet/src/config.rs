//! Test net configuration.

use crate::Arch;
use anyhow::Result;
use serde::Deserialize;
use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf, str::FromStr};

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

    /// The architecture of the node.
    pub arch: Arch,

    /// The path of the node specification.
    pub spec: PathBuf,

    /// The path of the node data.
    pub data: PathBuf,

    /// The QUIC address of the node.
    pub quic: String,

    /// The RPC address of the node.
    #[serde(default = "default::rpc")]
    pub rpc: String,

    /// The extra arguments for the node.
    #[serde(default)]
    pub args: Vec<String>,

    /// The validator seed of the node.
    pub seed: String,

    /// The environment variables for the node.
    pub env: BTreeMap<String, String>,
}

impl Node {
    /// Get the QUIC port of the node.
    pub fn quic_port(&self) -> Result<u16> {
        let addr = SocketAddr::from_str(&self.quic)?;
        Ok(addr.port())
    }

    /// Get the RPC port of the node.
    pub fn rpc_port(&self) -> Result<u16> {
        let addr = SocketAddr::from_str(&self.rpc)?;
        Ok(addr.port())
    }
}

mod default {
    /// The default RPC address.
    pub fn rpc() -> String {
        "0.0.0.0:0".to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::Testnet;

    const CONFIG: &str = r#"
    [node.alice]
    command = "spacejam"
    arch = "polkajam"
    spec = "alice.json"
    data = "alice"
    quic = "127.0.0.1:9944"
    rpc = "127.0.0.1:9933"
    args = []
    seed = "0"
    env = { RUST_LOG = "debug" }
    "#;

    #[test]
    fn parse_toml() {
        let testnet: Testnet = toml::from_str(CONFIG).unwrap();
        assert_eq!(testnet.node.len(), 1);
        assert_eq!(testnet.node.get("alice").unwrap().command, "spacejam");
    }
}
