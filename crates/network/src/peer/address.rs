//! Peer address.

use serde::{Deserialize, Serialize};

use crate::peer::PeerId;
use std::{fmt, net::SocketAddr, str::FromStr};

/// Peer address.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    /// Peer ID.
    pub peer_id: PeerId,

    /// Address.
    pub addr: SocketAddr,
}

impl Address {
    /// Create a new address.
    pub fn new(addr: SocketAddr, peer_id: impl Into<PeerId>) -> Self {
        Self {
            peer_id: peer_id.into(),
            addr,
        }
    }
}

impl<T: Into<PeerId>> From<(SocketAddr, T)> for Address {
    fn from(value: (SocketAddr, T)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.peer_id, self.addr)
    }
}

impl FromStr for Address {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (peer_id, addr) = s
            .split_once('@')
            .ok_or_else(|| anyhow::anyhow!("invalid address"))?;

        let peer_id = PeerId::from_str(peer_id)?;
        let addr = SocketAddr::from_str(addr)?;
        Ok(Self::new(addr, peer_id))
    }
}
