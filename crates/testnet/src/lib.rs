//! Testnet binary for running multiple nodes in a single process.

use serde::{Deserialize, Serialize};
pub use {
    cmd::App,
    config::{Network, Node, Testnet},
    log::Message,
};

mod cmd;
mod config;
mod log;
mod node;

/// A jam node that can be used in the testnet.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub enum Arch {
    /// The polkajam node.
    #[serde(rename = "polkajam")]
    #[default]
    Polkajam,

    /// The spacejam node.
    #[serde(rename = "spacejam")]
    SpaceJam,
}

impl Arch {
    /// Get the name of the arch.
    pub fn repo(&self) -> &str {
        match self {
            Arch::Polkajam => "https://github.com/paritytech/polkajam-releases/issues",
            Arch::SpaceJam => "https://github.com/spacejamapp/specjam/issues",
        }
    }
}
