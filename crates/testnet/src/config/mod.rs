//! Test net configuration.

use crate::Arch;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};
pub use {network::Network, node::Node};

mod network;
mod node;

#[derive(Serialize, Deserialize, Debug)]
pub struct Testnet {
    /// The network configuration.
    pub network: Network,

    /// The nodes in the testnet.
    pub node: BTreeMap<String, Node>,
}

impl Testnet {
    /// Prune the testnet.
    pub fn prune(&self) -> anyhow::Result<()> {
        for (name, node) in self.node.iter() {
            println!("pruning {name}: {}", node.data.display());
            fs::remove_dir_all(&node.data)?;
        }
        println!("pruned all nodes.");
        Ok(())
    }
}

impl Default for Testnet {
    fn default() -> Self {
        let mut nodes = BTreeMap::new();
        let quic = 40000;
        let rpc = 19800;

        for i in 0..5 {
            nodes.insert(
                format!("polkajam-{}", i),
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
            node: nodes,
            network: Network {
                spec: PathBuf::from("spec.json"),
                ..Default::default()
            },
        }
    }
}
