//! Node implementations.

use std::{future::Future, path::PathBuf};

mod polkajam;
mod spacejam;

/// A jam node that can be used in the testnet.
pub trait JamNode {
    /// Set the path of node specification.
    fn spec(&mut self, path: PathBuf) -> anyhow::Result<()>;

    /// Set the data path of node data.
    fn data(&mut self, path: PathBuf) -> anyhow::Result<()>;

    /// Set the QUIC address of the node.
    fn quic(&mut self, address: &str) -> anyhow::Result<()>;

    /// Set the RPC address of the node.
    fn rpc(&mut self, address: &str) -> anyhow::Result<()>;

    /// Set the extra arguments for the node.
    fn args(&mut self, args: Vec<String>) -> anyhow::Result<()>;

    /// Set the validator seed.
    fn seed(&mut self, seed: &str) -> anyhow::Result<()>;

    /// Start the node.
    fn start(self) -> impl Future<Output = anyhow::Result<()>> + Send;
}
