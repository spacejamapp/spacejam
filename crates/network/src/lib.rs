//! Network implementation of Spacejam.
#![allow(unused)]

use crypto::ed25519;
use rustls::crypto::hpke::EncapsulatedSecret;
use std::sync::Arc;
use tokio::sync::mpsc;
pub use {
    config::Config,
    context::Context,
    event::{Action, Event},
    peer::Address,
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

    /// Bootstrap peers
    bootstrap: Vec<Address>,
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

        Ok(Self {
            transport,
            erx,
            arx,
            bootstrap: config.bootstrap,
        })
    }

    /// Spawn the network
    pub async fn spawn<C: Context + Send + Sync + 'static>(mut self, context: Arc<C>) {
        let transport_spawn = self.transport.clone();
        let transport_bootstrap = self.transport.clone();
        let bootstrap = self.bootstrap.clone();
        transport_spawn.spawn(context.clone());

        // Spawn the event handling loop
        tokio::spawn(async move {
            let mut arx = self.arx;
            let mut erx = self.erx;

            loop {
                let ctx = context.clone();
                match tokio::select! {
                    Some(act) = arx.recv() => {
                        Ok::<Event, anyhow::Error>(Event::Action(act))
                    }
                    Some(ev) = erx.recv() => {
                        Ok::<Event, anyhow::Error>(ev)
                    }
                    else => {
                        tracing::debug!("all channels closed, terminating event loop");
                        break;
                    }
                } {
                    Ok(e) => e.handle_unchecked(ctx),
                    Err(e) => tracing::error!("event handling error: {e:?}"),
                }
            }
        });

        // Spawn a task to handle bootstrap dialing
        if !bootstrap.is_empty() {
            tokio::spawn(async move {
                tracing::debug!("dialing bootstrap peers: {:?}", bootstrap);
                for peer in bootstrap {
                    tracing::debug!("dialing bootstrap peer: {peer}");
                    if let Err(e) = transport_bootstrap.dial(peer).await {
                        tracing::warn!("failed to dial bootstrap peer: {e}");
                    }
                }
            });
        } else {
            tracing::debug!("no bootstrap peers, skipping");
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
