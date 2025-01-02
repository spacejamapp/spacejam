//! The configuration of SpaceJam network

use serde::{Deserialize, Serialize};
use std::{net::SocketAddrV4, path::PathBuf};

/// The configuration of SpaceJam network
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The configuration of the server
    pub server: ServerConfig,

    /// The configuration of the client
    pub client: ClientConfig,
}

/// The configuration of the client
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    /// The address of the client
    pub addr: SocketAddrV4,
}

/// The configuration of the server
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    /// The address of the server
    pub addr: SocketAddrV4,

    /// The server's X.509 certificate
    pub cert: PathBuf,

    /// The server's private key
    pub key: PathBuf,
}
