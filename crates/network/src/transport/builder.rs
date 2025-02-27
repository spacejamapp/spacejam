//! network builder.

use crate::{
    event,
    peer::PeerId,
    transport::{Transport, Verifier},
};
use crypto::ed25519;
use quinn::{
    crypto::rustls::{QuicClientConfig, QuicServerConfig},
    Endpoint,
};
use rcgen::CertificateParams;
use rustls::{
    crypto::{CryptoProvider, WebPkiSupportedAlgorithms},
    pki_types::PrivatePkcs8KeyDer,
    sign::{CertifiedKey, SingleCertAndKey},
    SignatureScheme,
};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::mpsc;
use webpki::types::CertificateDer;

/// Supported signature algorithms.
static SUPPORTED_SIG_ALGS: WebPkiSupportedAlgorithms = WebPkiSupportedAlgorithms {
    all: &[webpki::ring::ED25519],
    mapping: &[(SignatureScheme::ED25519, &[webpki::ring::ED25519])],
};

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
    pub fn build(self, tx: mpsc::UnboundedSender<event::peer::Event>) -> anyhow::Result<Transport> {
        let dns = PeerId::from(self.ed25519.verifying.as_bytes()).to_string();
        let provider = Self::provider();

        // setup cert
        let key = PrivatePkcs8KeyDer::from(self.ed25519.private_pkcs8_der()?);
        let keypair = rcgen::KeyPair::from_remote(Box::new(self.ed25519.clone()))?;
        let mut params = CertificateParams::new(vec![dns])?;

        // Set key usages for client and server auth
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
        ];
        params.extended_key_usages = vec![
            rcgen::ExtendedKeyUsagePurpose::ClientAuth,
            rcgen::ExtendedKeyUsagePurpose::ServerAuth,
        ];

        let cert = params.self_signed(&keypair)?;
        let cert_der = cert.der().clone();

        // Configure QUIC transport parameters
        let transport_config = Arc::new({
            let mut transport = quinn::TransportConfig::default();
            transport.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
            transport.max_idle_timeout(Some(quinn::IdleTimeout::from(quinn::VarInt::from_u32(
                10_000,
            ))));
            transport
        });

        // setup ALPN protocol
        let alpn = format!(
            "jamnp-s/{}/{}",
            self.version,
            &hex::encode(self.genesis)[..8]
        )
        .as_bytes()
        .to_vec();

        // client and server config setup
        let server = Self::server(
            alpn.clone(),
            cert_der.to_vec(),
            key.clone_key(),
            transport_config.clone(),
        )?;
        let client = Self::client(
            alpn,
            cert_der,
            key.clone_key(),
            provider,
            transport_config.clone(),
        )?;

        // setup endpoint
        let mut endpoint = Endpoint::server(server, self.address)?;
        endpoint.set_default_client_config(client);

        let address = crate::Address::new(
            self.address,
            PeerId::from(self.ed25519.verifying.as_bytes()),
        );
        tracing::info!("listening on {address}");
        Ok(Transport { endpoint, tx })
    }

    fn client(
        alpn: Vec<u8>,
        cert_der: CertificateDer<'static>,
        key: PrivatePkcs8KeyDer,
        provider: Arc<CryptoProvider>,
        transport_config: Arc<quinn::TransportConfig>,
    ) -> anyhow::Result<quinn::ClientConfig> {
        // Create our client certificate
        let single = SingleCertAndKey::from(CertifiedKey::from_der(
            vec![cert_der.clone()],
            key.clone_key().into(),
            &provider,
        )?);

        // Set up root certificate store - we trust all peer certificates
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add(cert_der.clone())?;

        // Create client config with our root store and client cert
        let mut crypto =
            rustls::ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
                .with_root_certificates(root_store)
                .with_client_cert_resolver(Arc::new(single));

        // Configure client to use ED25519 and our custom verifier
        crypto.alpn_protocols = vec![alpn];
        crypto.enable_early_data = true;
        crypto
            .dangerous()
            .set_certificate_verifier(Arc::new(Verifier));

        // Configure QUIC transport parameters for client
        let mut config = quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));
        config.transport_config(transport_config);
        Ok(config)
    }

    fn server(
        alpn: Vec<u8>,
        cert_der: Vec<u8>,
        key: PrivatePkcs8KeyDer,
        transport_config: Arc<quinn::TransportConfig>,
    ) -> anyhow::Result<quinn::ServerConfig> {
        // Create server config with our certificate and verifier
        let mut crypto =
            rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
                .with_client_cert_verifier(Arc::new(Verifier))
                .with_single_cert(vec![cert_der.clone().into()], key.clone_key().into())?;

        // Configure server to use ED25519
        crypto.alpn_protocols = vec![alpn];
        crypto.ignore_client_order = true;
        crypto.key_log = Arc::new(rustls::KeyLogFile::new());

        // Configure QUIC transport parameters for server
        let mut config =
            quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(crypto)?));
        config.transport_config(transport_config.clone());
        Ok(config)
    }

    fn provider() -> Arc<CryptoProvider> {
        let mut provider = rustls::crypto::ring::default_provider();
        provider.signature_verification_algorithms = SUPPORTED_SIG_ALGS;

        match provider.clone().install_default() {
            Ok(_) => Arc::new(provider),
            Err(e) => e,
        }
    }
}
