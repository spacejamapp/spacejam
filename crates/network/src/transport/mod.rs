//! Transport implementation for Spacejam.

use crate::{
    event::Event,
    peer::{Address, Connection, PeerId},
};
use anyhow::Context;
use crypto::ed25519;
use quinn::Endpoint;
use tokio::sync::{mpsc, oneshot};
pub use {builder::Builder, verifier::Verifier};

mod builder;
mod verifier;

/// Transport implementation for Spacejam.
#[derive(Clone)]
pub struct Transport {
    /// QUIC endpoint.
    pub(crate) endpoint: Endpoint,

    /// Event sender.
    pub tx: mpsc::UnboundedSender<Event>,
}

impl Transport {
    /// Create a new builder.
    pub fn builder(keypair: ed25519::KeyPair) -> builder::Builder {
        builder::Builder::new(keypair)
    }

    /// Dial a new connection.
    #[tracing::instrument(skip_all, fields(peer = %addr.peer_id))]
    pub async fn dial(&self, addr: Address) -> anyhow::Result<()> {
        tracing::debug!("dialing peer: {addr}");
        let conn = self
            .endpoint
            .connect(addr.addr, addr.peer_id.to_string().as_str())?
            .await
            .map_err(|_| anyhow::anyhow!("failed to dial {addr}"))?;

        // we need to verify the peer id before sending the connected event
        let Ok(conn) = Connection::new(conn.clone(), true) else {
            conn.close(1u32.into(), "failed to verify alpn".as_bytes());
            anyhow::bail!("failed to verify alpn of {addr}");
        };

        self.tx
            .send(Event::Connected { conn })
            .context("failed to send connected event")
    }

    /// Spawn new connections.
    pub fn spawn(self) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            if let Err(e) = tx.send(()) {
                tracing::warn!("failed to send spawn signal: {e:?}");
            }

            loop {
                let Some(conn) = self.endpoint.accept().await else {
                    tracing::error!("endpoint is closed");
                    break;
                };

                let Ok(conn) = conn
                    .await
                    .map_err(|e| tracing::warn!("failed to accept connection: {e:?}"))
                else {
                    continue;
                };

                let Ok(conn) = Connection::new(conn, false).map_err(|e| {
                    tracing::warn!("failed to verify alpn: {e:?}");
                }) else {
                    continue;
                };

                if let Err(e) = self.tx.send(Event::Connected { conn }) {
                    tracing::warn!("failed to send connected event: {e:?}");
                }
            }
        });

        rx
    }

    /// Close a connection
    pub async fn close(&self, peer: PeerId, reason: String) {
        if let Err(e) = self.tx.send(Event::Closed { peer, reason }) {
            tracing::error!("failed to send closed event to {}: {e}", peer);
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
