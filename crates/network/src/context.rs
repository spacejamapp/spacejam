//! Context for the network.

use crate::{stream, Event};
use anyhow::Result;
use crypto::ed25519;
use metrics::Metrics;
use score::{runtime::Grandpa, service::WorkReport, Block, OpaqueHash};
use tokio::sync::mpsc;

/// Context for the network.
pub trait Context: RuntimeApi {
    /// Get the keypair of the p2p network.
    fn keypair(&self) -> Option<ed25519::KeyPair> {
        None
    }

    /// Get the metrics of the node.
    fn metrics(&self) -> &Metrics;

    /// Announce the handshake message.
    fn grandpa(&self) -> Grandpa;

    /// Get the event sender of the network.
    fn tx(&self) -> mpsc::UnboundedSender<Event>;
}

/// API for the network.
pub trait RuntimeApi {
    /// Fetch blocks from storage.
    fn fetch_blocks(&self, _request: stream::ce128::Request) -> Result<Vec<Block>> {
        Ok(Default::default())
    }

    /// Fetch state from storage.
    fn fetch_state(&self, _request: stream::ce129::Request) -> Result<stream::ce129::Response> {
        Ok(Default::default())
    }

    /// Fetch a work report from storage.
    fn fetch_work_report(&self, _hash: OpaqueHash) -> Result<WorkReport> {
        Ok(Default::default())
    }

    /// Fetch a preimage from storage.
    fn fetch_preimage(&self, _hash: OpaqueHash) -> Result<Vec<u8>> {
        Ok(Default::default())
    }
}
