//! network builder.

use crate::{
    transport::{Transport, Verifier},
    Action, Event,
};
use crypto::ed25519;
use quinn::{
    crypto::rustls::{QuicClientConfig, QuicServerConfig},
    Endpoint,
};
use rcgen::CertificateParams;
use rustls::{
    pki_types::PrivatePkcs8KeyDer,
    sign::{CertifiedKey, SingleCertAndKey},
};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::mpsc;
use webpki::types::pem::PemObject;

/// Network builder.
pub struct Builder {
    /// Socket address.
    address: SocketAddr,

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
            address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            genesis: [0; 32],
            version: "0".to_string(),
        }
    }

    /// Set the socket address.
    pub fn address(mut self, address: SocketAddr) -> Self {
        self.address = address;
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
    pub fn build(self, tx: mpsc::UnboundedSender<Event>) -> anyhow::Result<Transport> {
        let dns = format!(
            "e{}",
            base32::encode(
                base32::Alphabet::Rfc4648Lower { padding: false },
                self.ed25519.verifying.as_bytes(),
            )
        );
        tracing::info!("dns_name: {dns}");

        // setup provider
        let provider = rustls::crypto::ring::default_provider();
        let provider = match provider.clone().install_default() {
            Ok(_) => Arc::new(provider),
            Err(e) => e,
        };

        // setup cert
        let key = PrivatePkcs8KeyDer::from(self.ed25519.private_pkcs8_der()?);
        let keypair = rcgen::KeyPair::from_remote(Box::new(self.ed25519))?;
        let cert = CertificateParams::new(vec![dns])?.self_signed(&keypair)?;
        let cert_der = cert.der().clone();

        // server config setup
        let server = {
            let crypto =
                rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
                    .with_client_cert_verifier(Arc::new(Verifier))
                    .with_single_cert(vec![cert.into()], key.clone_key().into())?;

            quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(crypto)?))
        };

        // client config setup
        let client = {
            let single = SingleCertAndKey::from(CertifiedKey::from_der(
                vec![cert_der],
                key.into(),
                &provider,
            )?);

            let crypto =
                rustls::ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
                    .with_root_certificates(rustls::RootCertStore::empty())
                    .with_client_cert_resolver(Arc::new(single));

            quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?))
        };

        // setup endpoint
        let mut endpoint = Endpoint::server(server, self.address)?;
        endpoint.set_default_client_config(client);

        tracing::info!("listening on {}:{}", self.address.ip(), self.address.port());
        Ok(Transport { endpoint, tx })
    }
}
