//! Test net configuration.

use crate::Arch;
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf};
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

impl Default for Testnet {
    fn default() -> Self {
        let mut nodes = BTreeMap::new();
        let quic = 40000;
        let rpc = 19800;

        for i in 0..4 {
            nodes.insert(
                "polkajam-0".to_string(),
                Node {
                    command: "polkajam".to_string(),
                    arch: Arch::Polkajam,
                    data: PathBuf::from(format!("res/testnet/{}", i)),
                    quic: format!("0.0.0.0:{}", quic + i),
                    rpc: format!("0.0.0.0:{}", rpc + i),
                    args: vec![],
                    seed: "0".to_string(),
                    env: BTreeMap::new(),
                    filter: vec![],
                },
            );
        }
        nodes.insert(
            "spacejam".to_string(),
            Node {
                command: "spacejam".to_string(),
                arch: Arch::SpaceJam,
                data: PathBuf::from("res/testnet/5"),
                quic: "0.0.0.0:40005".to_string(),
                rpc: "0.0.0.0:19805".to_string(),
                args: vec![],
                seed: "0".to_string(),
                env: BTreeMap::new(),
                filter: vec![],
            },
        );

        Self {
            node: BTreeMap::new(),
            network: Network {
                spec: PathBuf::from("spec.json"),
                ..Default::default()
            },
        }
    }
}
