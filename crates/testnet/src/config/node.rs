//! Node configuration.

use crate::{config::Filter, Arch};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf, str::FromStr};

/// A node in the testnet.
#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    /// The binary name of the node.
    pub command: String,

    /// The architecture of the node.
    pub arch: Arch,

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

    /// The log filters of the node.
    #[serde(default, flatten)]
    pub filter: Filter,
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
