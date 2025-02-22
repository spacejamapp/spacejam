//! Transport implementation for Spacejam.

use crate::event::{peer, Action, Event};
use crypto::ed25519;
use quinn::{crypto::rustls::HandshakeData, Endpoint};
use rcgen::Certificate;
use tokio::sync::mpsc;
use webpki::{types::CertificateDer, EndEntityCert};
pub use {builder::Builder, verifier::Verifier};

mod builder;
mod verifier;

/// Transport implementation for Spacejam.
pub struct Transport {
    /// QUIC endpoint.
    pub(crate) endpoint: Endpoint,

    /// Event sender.
    pub(crate) tx: mpsc::UnboundedSender<Event>,

    /// Action receiver.
    pub(crate) rx: mpsc::UnboundedReceiver<Action>,
}

impl Transport {
    /// Create a new builder.
    pub fn builder(keypair: ed25519::KeyPair) -> builder::Builder {
        builder::Builder::new(keypair)
    }

    /// Accept a new connection.
    ///
    /// TODO: verify all of usages of downcasts in the process. this reuqires tests
    pub async fn accept(&self) -> anyhow::Result<Event> {
        let conn = self
            .endpoint
            .accept()
            .await
            .ok_or_else(|| anyhow::anyhow!("endpoint is closed"))?
            .await?;

        self::alpn(&conn)?;

        let peer = self::peer(&conn)?;
        Ok(peer::Event::ConnectionEstablished { peer }.into())
    }
}

/// Get the peer from the Connection
///
/// Note that the DNS name should be verified by the Verifier.
pub fn peer(conn: &quinn::Connection) -> anyhow::Result<[u8; 32]> {
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

/// Verify ALPN after accepting a connection.
pub fn alpn(conn: &quinn::Connection) -> anyhow::Result<()> {
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

    Ok(())
}
