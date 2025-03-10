//! Peer connection.

use crate::{
    peer::{Address, PeerId},
    stream::up0::Handshake,
    transport::Verifier,
};
use quinn::crypto::rustls::HandshakeData;
use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::RwLock;
use webpki::{types::CertificateDer, EndEntityCert};

/// Peer connection.
#[derive(Debug, Clone)]
pub struct Connection {
    /// Peer ID.
    pub address: Address,

    /// QUIC connection.
    pub conn: quinn::Connection,

    /// Handshake data.
    pub handshake: Arc<RwLock<Handshake>>,

    /// Whether the connection is ready.
    pub ready: Arc<AtomicBool>,

    /// Latency.
    pub latency: Duration,

    /// Direction.
    pub outgoing: bool,
}

impl Connection {
    /// Create a new connection.
    pub fn new(conn: quinn::Connection, outgoing: bool) -> anyhow::Result<Self> {
        let peer = self::alpn(&conn)?;
        let address = Address::new(conn.remote_address(), peer);

        Ok(Self {
            address,
            conn,
            handshake: Arc::new(RwLock::new(Handshake::default())),
            ready: Arc::new(AtomicBool::new(false)),
            latency: Duration::from_secs(0),
            outgoing,
        })
    }

    /// Check if the connection is ready.
    pub fn ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }
}

impl Deref for Connection {
    type Target = quinn::Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

/// Verify ALPN after accepting a connection.
fn alpn(conn: &quinn::Connection) -> anyhow::Result<PeerId> {
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
fn peer(conn: &quinn::Connection) -> anyhow::Result<PeerId> {
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

    Verifier::extract_public_key(&cert)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .map(Into::into)
}
