//! Transport implementation for Spacejam.

use crate::{
    event::{peer, Action, Event},
    peer::Address,
    stream::Stream,
    Context,
};
use crypto::ed25519;
use quinn::{crypto::rustls::HandshakeData, Connection, Endpoint};
use rcgen::Certificate;
use std::{net::SocketAddr, sync::Arc};
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
    #[tracing::instrument(skip(self))]
    pub async fn dial(&self, addr: Address) -> anyhow::Result<Connection> {
        tracing::debug!("dialing {addr}");
        let connecting = self.endpoint.connect(addr.addr, addr.peer_id.as_ref())?;
        tracing::debug!(
            "initiated connection to {}, waiting for completion",
            addr.addr
        );
        connecting
            .await
            .map_err(|e| anyhow::anyhow!("failed to dial {addr}: {e:?}"))
    }

    /// Spawn a new connection.
    pub async fn spawn(self) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            tx.send(());
            while let Some(conn) = self.accept().await {
                tracing::debug!("accepted connection from {:?}", conn.remote_address());
                let Ok(peer) = self::alpn(&conn).map_err(|e| {
                    tracing::warn!("failed to verify ALPN: {e:?}");
                }) else {
                    continue;
                };

                self.tx.send(peer::Event::Connected { peer }.into());

                // handle connection
                let this = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = this.handle(conn).await {
                        tracing::warn!("failed to handle connection: {e:?}");
                        this.tx.send(peer::Event::Closed { peer }.into());
                    }
                });
            }

            tracing::warn!("transport handler exited");
        });

        rx.await
            .map_err(|_| anyhow::anyhow!("failed to spawn transport handler"))
    }

    /// Handle a new connection.
    ///
    /// note that all communication happens over bidirectional QUIC streams.
    async fn handle(&self, conn: quinn::Connection) -> anyhow::Result<()> {
        tracing::debug!("handling connection from {:?}", conn.remote_address());
        while let Ok((send, mut recv)) = conn.accept_bi().await {
            let stream_id = send.id();
            tracing::debug!("accepted bi-directional stream {}", stream_id);

            // Read stream type byte.
            //
            // e.g. after opening a stream, the stream initiator must send a single
            // byte identifying the stream kind.
            let mut buf = [0u8; 1];
            recv.read_exact(&mut buf).await?;
            let stream_type = Stream::from(buf[0]);
            tracing::debug!("received stream type: {:?}", stream_type);
        }

        Ok(())
    }

    /// Accept a new connection.
    ///
    /// TODO: handle retrying from incoming.
    async fn accept(&self) -> Option<Connection> {
        tracing::debug!("waiting for incoming connection");
        let incoming = match self.endpoint.accept().await {
            Some(conn) => {
                tracing::debug!("received incoming connection request");
                conn
            }
            None => {
                tracing::warn!("endpoint.accept() returned None");
                return None;
            }
        };

        tracing::debug!("waiting for connection completion");
        match incoming.await {
            Ok(conn) => {
                tracing::debug!(
                    "connection accepted successfully from {:?}",
                    conn.remote_address()
                );
                Some(conn)
            }
            Err(e) => {
                tracing::warn!("failed to accept connection: {e:?}");
                None
            }
        }
    }
}

/// Verify ALPN after accepting a connection.
fn alpn(conn: &quinn::Connection) -> anyhow::Result<[u8; 32]> {
    let data: Box<HandshakeData> = conn
        .handshake_data()
        .ok_or_else(|| rustls::Error::HandshakeNotComplete)?
        .downcast()
        .map_err(|_| anyhow::anyhow!("invalid handshake data"))?;

    // TODO: identify what exactly the server name is.
    if let Some(server_name) = data.server_name {
        tracing::debug!("handshake from server name: {:?}", server_name);
    }

    let protocol = String::from_utf8(
        data.protocol
            .ok_or_else(|| anyhow::anyhow!("none protocol in handshake data"))?,
    )
    .map_err(|_| anyhow::anyhow!("could not parse protocol"))?;

    tracing::debug!("verifying ALPN protocol: {}", protocol);
    let patts = protocol.split('/').collect::<Vec<&str>>();
    let patts_len = patts.len();

    // jamnp-s/V/H
    // jamnp-s/V/H/builder
    if !(2..=4).contains(&patts_len) {
        tracing::warn!("invalid protocol pattern length: {}", patts_len);
        anyhow::bail!("invalid protocol patterns")
    }

    if patts[0] != crate::PROTOCOL {
        tracing::warn!("invalid protocol name: {}", patts[0]);
        anyhow::bail!("invalid protocol name");
    }

    if patts_len != 3 && patts[patts_len.saturating_sub(1)] != "builder" {
        tracing::warn!("invalid builder pattern");
        anyhow::bail!("invalid builder pattern");
    }

    tracing::debug!("ALPN protocol verified successfully");
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

    let mut bytes = [0; 32];
    bytes.copy_from_slice(&cert.subject_public_key_info());
    Ok(bytes)
}
