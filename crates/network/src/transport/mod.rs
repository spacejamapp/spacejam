//! Transport implementation for Spacejam.

use crate::{
    event::{peer, Action, Event},
    Context,
};
use crypto::ed25519;
use quinn::{crypto::rustls::HandshakeData, Connection, Endpoint};
use rcgen::Certificate;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
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
    pub async fn dial(&self, addr: SocketAddr, name: &str) -> anyhow::Result<Connection> {
        self.endpoint
            .connect(addr, name)?
            .await
            .map_err(|e| anyhow::anyhow!("failed to dial {addr}: {e:?}"))
    }

    /// Spawn a new connection.
    pub fn spawn<C: Context + Send + Sync + 'static>(self, ctx: Arc<C>) {
        tokio::spawn(async move {
            while let Some(conn) = self.accept().await {
                tracing::trace!("accepted connection");
                let Ok(peer) = self::alpn(&conn).map_err(|e| {
                    tracing::warn!("failed to verify ALPN: {e:?}");
                }) else {
                    continue;
                };

                self.tx.send(peer::Event::Connected { peer }.into());

                // handle connection
                let this = self.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    if let Err(e) = this.handle(conn, ctx.clone()).await {
                        tracing::warn!("failed to handle connection: {e:?}");
                        this.tx.send(peer::Event::Closed { peer }.into());
                    }
                });
            }
        });
    }

    /// Handle a new connection.
    ///
    /// note that all communication happens over bidirectional QUIC streams.
    async fn handle<C: Context>(&self, conn: quinn::Connection, ctx: Arc<C>) -> anyhow::Result<()> {
        while let Ok((send, mut recv)) = conn.accept_bi().await {
            let stream_id = send.id();

            // Read stream type byte.
            //
            // e.g. after opening a stream, the stream initiator must send a single
            // byte identifying the stream kind.
            let mut buf = [0u8; 1];
            recv.read_exact(&mut buf).await?;
            // let stream_type = StreamType::from(buf[0]);
        }

        Ok(())
    }

    /// Accept a new connection.
    ///
    /// TODO: handle retrying from incoming.
    async fn accept(&self) -> Option<Connection> {
        match self.endpoint.accept().await?.await {
            Ok(conn) => Some(conn),
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
        tracing::trace!("handshake from: {:?}", server_name);
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
        anyhow::bail!("invalid protocol patterns")
    }

    if patts[0] != crate::PROTOCOL {
        anyhow::bail!("invalid protocol name");
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

    let mut bytes = [0; 32];
    bytes.copy_from_slice(&cert.subject_public_key_info());
    Ok(bytes)
}
