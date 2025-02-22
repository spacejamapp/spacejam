//! Configuration for the network.

use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

/// Configuration for the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "cmd", derive(clap::Parser))]
pub struct Config {
    /// The address to listen on.
    #[cfg_attr(feature = "cmd", arg(long, default_value = "0.0.0.0:0"))]
    pub address: SocketAddr,

    /// The bootstrap addresses.
    #[cfg_attr(feature = "cmd", arg(long))]
    pub bootstrap: Vec<SocketAddr>,

    /// The genesis hash.
    ///
    /// This should be overriden by the genesis file.
    #[cfg_attr(feature = "cmd", clap(skip))]
    pub genesis: [u8; 32],
    // TODO: some other network configs.
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            bootstrap: vec![],
            genesis: [0; 32],
        }
    }
}
