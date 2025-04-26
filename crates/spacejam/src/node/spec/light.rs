//! Light node implementation
//!
//! TODO: introduce hook that stores blocks in the database

use crate::{node::spec::NodeSpec, offchain::Offchain};
use network::Network;
use std::net::SocketAddr;

/// Importing and finalizing blocks with grandpa with JSON-RPC provided
pub struct Light<C: runtime::Config> {
    pub(crate) network: Network<C>,
    pub(crate) rpc: SocketAddr,
    pub(crate) metrics: SocketAddr,
}

impl<C: runtime::Config> NodeSpec for Light<C> {
    async fn start(self) -> anyhow::Result<()> {
        let runtime = self.network.clone();
        let offchain = Offchain::new(runtime.runtime.clone());

        tokio::select! {
            _ = offchain.start(self.rpc, self.network.metrics.clone(), self.metrics) => {}
            _ = runtime.spawn() => {}
            _ = tokio::signal::ctrl_c() => {}
        }

        Ok(())
    }
}
