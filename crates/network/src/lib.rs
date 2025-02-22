//! Network implementation of Spacejam.
#![allow(unused)]

use crypto::ed25519;
use tokio::sync::mpsc;
pub use {
    config::Config,
    context::Context,
    event::{Action, Event},
    transport::{Builder as TransportBuilder, Transport},
};

mod config;
mod context;
mod event;
mod transport;

/// The network protocol name of Spacejam.
pub const PROTOCOL: &str = "jamnp-s";

/// Network implementation of Spacejam.
pub struct Network {
    transport: Transport,
    rx: mpsc::UnboundedReceiver<Event>,
}

impl Network {
    /// Create a new network.
    ///
    /// If the provided keypair is not provided, the node will not
    /// be a validator.
    pub async fn new(
        config: Config,
        arx: mpsc::UnboundedReceiver<Action>,
        keypair: Option<ed25519::KeyPair>,
    ) -> anyhow::Result<Self> {
        let (etx, erx) = mpsc::unbounded_channel();
        let transport = Transport::builder(keypair.unwrap_or_default())
            .address(config.address)
            .genesis(config.genesis)
            .build(etx, arx)?;

        Ok(Self { transport, rx: erx })
    }

    /// Spawn the network
    pub async fn spawn(&mut self, context: &impl Context) -> anyhow::Result<()> {
        loop {
            match tokio::select! {
                e = self.transport.accept() => e,
                act = self.rx.recv() => act.ok_or_else(|| anyhow::anyhow!("Local channel closed")),
            } {
                Ok(e) => e.handle(context)?,
                Err(e) => tracing::error!("{e:?}"),
            }
        }
    }
}
