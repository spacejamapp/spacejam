//! Network implementation of Spacejam.
#![allow(unused)]

use crypto::ed25519;
use std::sync::Arc;
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
pub mod peer;
mod stream;
mod transport;

/// The network protocol name of Spacejam.
pub const PROTOCOL: &str = "jamnp-s";

/// Network implementation of Spacejam.
pub struct Network {
    transport: Transport,

    /// Event receiver
    erx: mpsc::UnboundedReceiver<Event>,

    /// Action receiver
    arx: mpsc::UnboundedReceiver<Action>,
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
            .build(etx)?;

        // Dial bootstrap peers.
        //
        // TODO: dial bootstrap peers with names, the bootstrap peers should be
        // able found from the genesis config.
        for peer in config.bootstrap {
            tracing::trace!("dialing bootstrap peer: {peer}");
            transport.dial(peer).await?;
        }

        Ok(Self {
            transport,
            erx,
            arx,
        })
    }

    /// Spawn the network
    pub async fn spawn<C: Context + Send + Sync + 'static>(mut self, context: Arc<C>) {
        self.transport.spawn(context.clone());

        loop {
            let ctx = context.clone();
            match tokio::select! {
                act = self.arx.recv() => act.map(Into::into).ok_or_else(|| anyhow::anyhow!("Local action channel closed")),
                ev = self.erx.recv() => ev.ok_or_else(|| anyhow::anyhow!("Local event channel closed")),
            } {
                Ok(e) => e.handle_unchecked(ctx),
                Err(e) => tracing::error!("{e:?}"),
            }
        }
    }
}

/// Pick a random port.
pub fn pick() -> std::io::Result<u16> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let addr = socket.local_addr()?;
    Ok(addr.port())
}
