//! The configuration of SpaceJam network

use anyhow::Result;
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use serde::Deserialize;
use std::{
    fs,
    net::SocketAddrV4,
    path::{Path, PathBuf},
};

/// The configuration of SpaceJam network
#[derive(Debug, Deserialize)]
pub struct Config {
    /// The configuration of the server
    pub server: ServerConfig,

    /// The configuration of the client
    pub client: ClientConfig,
}

impl TryFrom<&Path> for Config {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let config = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file at {path:?}: {e}"))?;

        Ok(toml::from_str(&config)?)
    }
}

/// The configuration of the client
#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    /// The address of the client
    pub addr: SocketAddrV4,

    /// The configuration of the DER certificates
    #[serde(flatten)]
    pub der: DerConfig,
}

/// The configuration of the server
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// The address of the server
    pub addr: SocketAddrV4,

    /// The configuration of the DER certificates
    #[serde(flatten)]
    pub der: DerConfig,
}

/// The configuration of the DER certificates
#[derive(Debug, Deserialize, Clone)]
pub struct DerConfig {
    /// The X.509 certificates
    pub cert: Vec<PathBuf>,

    /// The private key
    pub key: PathBuf,
}

impl DerConfig {
    /// Load the certificates and private key
    pub fn load(&self) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        TryFrom::try_from(self)
    }
}

impl TryFrom<&DerConfig> for (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>) {
    type Error = anyhow::Error;

    fn try_from(
        config: &DerConfig,
    ) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        let certs = config
            .cert
            .iter()
            .map(|cert| {
                CertificateDer::from_pem_file(cert)
                    .map_err(|e| anyhow::anyhow!("Failed to read certificate file {:?}: {e}", cert))
            })
            .collect::<Result<Vec<_>>>()?;

        let key = PrivateKeyDer::from_pem_file(&config.key)
            .map_err(|e| anyhow::anyhow!("Failed to read key file {:?}: {e}", config.key))?;

        Ok((certs, key))
    }
}
