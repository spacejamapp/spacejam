//! Network implementation of Spacejam.
#![allow(unused)]

use crypto::ed25519;
use tokio::sync::mpsc;
pub use {
    config::Config,
    context::Context,
    event::Event,
    transport::{Builder as TransportBuilder, Transport},
};

mod config;
mod context;
mod event;
mod transport;

/// Network implementation of Spacejam.
pub struct Network {
    transport: Transport,
}

impl Network {
    /// Create a new network.
    pub async fn new(
        _config: Config,
        _rx: mpsc::Receiver<Event>,
        keypair: Option<ed25519::KeyPair>,
    ) -> anyhow::Result<Self> {
        let transport = Transport::builder(keypair.expect("keypair is required"))
            .build()
            .unwrap();
        Ok(Self { transport })
    }

    /// Spawn the network
    pub async fn spawn(&self, context: &impl Context) -> anyhow::Result<()> {
        Ok(())
    }
}
