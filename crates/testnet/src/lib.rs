//! Testnet binary for running multiple nodes in a single process.

pub use config::{Node, Testnet};
use serde::{Deserialize, Serialize};

mod config;
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
