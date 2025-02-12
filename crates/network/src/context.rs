//! Context for the network.

use anyhow::Result;
use litep2p::crypto::ed25519;
use metrics::Metrics;

/// Context for the network.
pub trait Context {
    /// Get the metrics of the node.
    fn metrics(&self) -> &Metrics;

    /// Get the keypair of the p2p network.
    fn keypair(&self) -> Option<ed25519::Keypair> {
        None
    }

    /// Import a block.
    ///
    /// This method will be called when receiving a block
    /// announcement from a peer.
    fn import_block(&self, _: Vec<u8>) -> Result<()> {
        Ok(())
    }

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

    /// Subscribe a ticket.
    ///
    /// NOTE: This should be called outside from the network module.
    fn subscribe_ticket(&self, _ticket: Vec<u8>) -> Result<()> {
        Ok(())
    }
}

impl Context for Metrics {
    fn metrics(&self) -> &Metrics {
        self
    }
}
