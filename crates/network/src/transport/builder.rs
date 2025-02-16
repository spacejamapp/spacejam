//! network builder.

use crate::transport::Verifier;
use crypto::ed25519;
use quinn::{crypto::rustls::QuicServerConfig, Endpoint};
use rcgen::CertificateParams;
use rustls::pki_types::PrivatePkcs8KeyDer;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use super::Transport;

/// Network builder.
pub struct Builder {
    /// Socket address.
    addr: SocketAddr,

    /// Ed25519 key pair.
    ed25519: ed25519::KeyPair,

    /// The genesis hash.
    genesis: [u8; 32],

    /// The protocol version.
    version: String,
}

impl Builder {
    /// Create a new builder.
    pub fn new(ed25519: ed25519::KeyPair) -> Self {
        Self {
            ed25519,
            addr: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            genesis: [0; 32],
            version: "0".to_string(),
        }
    }

    /// Set the socket address.
    pub fn addr(mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
        self
    }

    /// Set the genesis hash.
    pub fn genesis(mut self, hash: [u8; 32]) -> Self {
        self.genesis = hash;
        self
    }

    /// Set the protocol version.
    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Build the QUIC server.
    pub fn build(self) -> anyhow::Result<Transport> {
        let dns = format!(
            "e{}",
            base32::encode(
                base32::Alphabet::Rfc4648Lower { padding: false },
                self.ed25519.verifying.as_bytes(),
            )
        );

        let keypair = rcgen::KeyPair::from_remote(Box::new(self.ed25519))?;
        let cert = CertificateParams::new(vec![dns])?.self_signed(&keypair)?;
        let key = PrivatePkcs8KeyDer::from(keypair.serialize_der());
        let crypto =
            rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
                .with_client_cert_verifier(Arc::new(Verifier))
                .with_single_cert(vec![cert.into()], key.into())?;

        let server =
            quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(crypto)?));
        Ok(Transport {
            endpoint: Endpoint::server(server, self.addr)?,
        })
    }
}
