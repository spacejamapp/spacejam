//! The offchain components of SpaceJam

pub use hook::OffchainHook;
use runtime::Runtime;
use std::{net::SocketAddr, sync::Arc};

mod hook;
pub mod service;

/// The entypoint of the offchain services
pub struct Offchain<C: runtime::Config> {
    /// The RPC server
    pub rpc: service::Rpc<C>,
}

impl<C: runtime::Config> Offchain<C> {
    /// Create a new offchain component
    pub fn new(runtime: Arc<Runtime<C>>) -> Self {
        Self {
            rpc: service::Rpc::new(runtime),
        }
    }

    /// Get the hook
    pub fn hook(&self) -> OffchainHook<C> {
        OffchainHook::new(self.rpc.clone())
    }

    /// Start the offchain services
    pub async fn start(self, rpc: SocketAddr) -> anyhow::Result<()> {
        self.rpc.start(rpc).await?;
        Ok(())
    }
}
