//! Configuration for the network.

use crate::peer::{Address, PeerId};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

/// Configuration for the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "cmd", derive(clap::Parser))]
pub struct Config {
    /// The address to listen on.
    #[cfg_attr(feature = "cmd", arg(long, default_value = "0.0.0.0:0"))]
    #[serde(alias = "listen-ip")]
    pub address: SocketAddr,

    /// The peer id.
    #[cfg_attr(feature = "cmd", arg(long))]
    pub peer_id: Option<PeerId>,

    /// The bootstrap address.
    #[cfg_attr(feature = "cmd", arg(long))]
    pub bootnode: Option<Address>,

    /// The genesis hash.
    ///
    /// This should be overridden by the genesis file.
    #[cfg_attr(feature = "cmd", clap(skip))]
    pub genesis: [u8; 32],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            peer_id: None,
            bootnode: None,
            genesis: [0; 32],
        }
    }
}
