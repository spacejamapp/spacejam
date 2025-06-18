//! Testnet binary for running multiple nodes in a single process.

pub use {
    config::{Node, Testnet},
    node::JamNode,
};

mod config;
mod node;
