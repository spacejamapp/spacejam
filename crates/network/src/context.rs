//! Context for the network.

use anyhow::Result;
use litep2p::crypto::ed25519;

/// Context for the network.
pub trait Context {
    /// Get the keypair of the p2p network.
    fn keypair(&self) -> Option<ed25519::Keypair> {
        None
    }

    /// Import a block.
    ///
    /// This method will be called when receiving a block
    /// announcement from a peer.
    fn import_block(&self, block: Vec<u8>) -> Result<()>;

    /// Finalize a block.
    ///
    /// TODO:
    ///
    /// 1. when to finalize.
    /// 2. which module will call this?
    fn finalize_block(&self, _block: Vec<u8>) -> Result<()> {
        Ok(())
    }

    /// Subscribe a block.
    ///
    /// NOTE: This should be called outside from the network module.
    ///
    /// TODO:
    ///
    /// 1. when to subscribe.
    /// 2. which module will call this?
    fn subscribe_block(&self, _block: Vec<u8>) -> Result<()> {
        Ok(())
    }
}

impl Context for () {
    fn import_block(&self, _block: Vec<u8>) -> Result<()> {
        Ok(())
    }
}
