//! The offchain components of SpaceJam

use ::rpc::{ApiServer, Server};
use rpc::Rpc;
use runtime::Runtime;
use std::{net::SocketAddr, sync::Arc};

mod rpc;

/// The entypoint of the offchain services
pub struct Offchain<C: runtime::Config> {
    /// The RPC server
    pub rpc: Rpc<C>,
}

impl<C: runtime::Config> Offchain<C> {
    /// Create a new offchain component
    pub fn new(runtime: Arc<Runtime<C>>) -> Self {
        Self {
            rpc: Rpc::new(runtime),
        }
    }

    /// Start the offchain services
    pub async fn start(self, rpc: SocketAddr) -> anyhow::Result<()> {
        let server = Server::builder().build(rpc).await?;
        let addr = server.local_addr()?;
        tracing::info!("Listening RPC on {}", addr);
        server.start(self.rpc.into_rpc()).stopped().await;
        Ok(())
    }
}
