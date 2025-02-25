//! Transport implementation for Spacejam.

use crate::{
    event::{peer, Event},
    peer::{Address, PeerId},
};
use anyhow::Context;
use crypto::ed25519;
use quinn::{crypto::rustls::HandshakeData, Endpoint};
use tokio::sync::{mpsc, oneshot};
use webpki::{types::CertificateDer, EndEntityCert};
pub use {builder::Builder, verifier::Verifier};

mod builder;
mod verifier;

/// Transport implementation for Spacejam.
#[derive(Clone)]
pub struct Transport {
    /// QUIC endpoint.
    pub(crate) endpoint: Endpoint,

    /// Event sender.
    pub(crate) tx: mpsc::UnboundedSender<Event>,
}

impl Transport {
    /// Create a new builder.
    pub fn builder(keypair: ed25519::KeyPair) -> builder::Builder {
        builder::Builder::new(keypair)
    }

    /// Dial a new connection.
    #[tracing::instrument(skip_all, fields(peer = addr.peer_id.to_string()))]
    pub async fn dial(&self, addr: Address) -> anyhow::Result<()> {
        let conn = self
            .endpoint
            .connect(addr.addr, addr.peer_id.as_ref())?
            .await
            .map_err(|_| anyhow::anyhow!("failed to dial {addr}"))?;

        self.tx
            .send(Event::Peer(peer::Event::Connected {
                peer: self::alpn(&conn).context("failed to verify alpn")?,
                connection: conn,
            }))
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

                let Ok(peer) = self::alpn(&conn).map_err(|e| {
                    tracing::warn!("failed to verify alpn: {e:?}");
                }) else {
                    continue;
                };

                if let Err(e) = self.tx.send(Event::Peer(peer::Event::Connected {
                    peer,
                    connection: conn,
                })) {
                    tracing::warn!("failed to send connected event: {e:?}");
                }
            }
        });

        rx
    }
}

/// Verify ALPN after accepting a connection.
fn alpn(conn: &quinn::Connection) -> anyhow::Result<[u8; 32]> {
    let data: Box<HandshakeData> = conn
        .handshake_data()
        .ok_or_else(|| rustls::Error::HandshakeNotComplete)?
        .downcast()
        .map_err(|_| anyhow::anyhow!("invalid handshake data"))?;

    // validate the server name
    if let Some(alt) = data.server_name {
        let _ = PeerId::verify(&alt)?;
    }

    let protocol = String::from_utf8(
        data.protocol
            .ok_or_else(|| anyhow::anyhow!("none protocol in handshake data"))?,
    )
    .map_err(|_| anyhow::anyhow!("could not parse protocol"))?;

    let patts = protocol.split('/').collect::<Vec<&str>>();
    let patts_len = patts.len();

    // jamnp-s/V/H
    // jamnp-s/V/H/builder
    if !(2..=4).contains(&patts_len) {
        anyhow::bail!("invalid protocol pattern length: {patts_len}")
    }

    if patts[0] != crate::PROTOCOL {
        anyhow::bail!("invalid protocol name: {}", patts[0]);
    }

    if patts_len != 3 && patts[patts_len.saturating_sub(1)] != "builder" {
        anyhow::bail!("invalid builder pattern");
    }

    self::peer(conn)
}

/// Get the peer from the Connection
///
/// Note that the DNS name should be verified by the Verifier.
fn peer(conn: &quinn::Connection) -> anyhow::Result<[u8; 32]> {
    let Some(identity) = conn.peer_identity() else {
        anyhow::bail!("no peer identity");
    };

    let certs: Box<Vec<CertificateDer<'_>>> = identity
        .downcast()
        .map_err(|_| anyhow::anyhow!("invalid peer identity"))?;

    let cert = certs
        .first()
        .ok_or_else(|| anyhow::anyhow!("no certs found from identity"))?;

    let cert =
        EndEntityCert::try_from(cert).map_err(|_| anyhow::anyhow!("invalid peer identity"))?;

    Verifier::extract_public_key(&cert).map_err(|e| anyhow::anyhow!("{e:?}"))
}
