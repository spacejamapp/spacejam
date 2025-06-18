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
#[derive(Deserialize, Serialize)]
pub enum Arch {
    /// The polkajam node.
    #[serde(alias = "polkajam")]
    Polkajam,

    /// The spacejam node.
    #[serde(alias = "spacejam")]
    SpaceJam,
}
