//! Context for the network.

use anyhow::Result;

/// Context for the network.
pub trait Context {
    /// Import a block.
    ///
    /// This method will be called when receiving a block
    /// announcement from a peer.
    fn import_block(&self, block: Vec<u8>) -> Result<()>;
}
