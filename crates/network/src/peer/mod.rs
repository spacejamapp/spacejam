//! Peer related stuffs

use score::OpaqueHash;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
pub use {address::Address, conn::Connection};

mod address;
mod conn;
mod format;

/// Peer ID, also known as the DNS name of the peer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct PeerId(OpaqueHash);

impl PeerId {
    /// Create a new peer id.
    pub fn verify(id: &str) -> Result<Self, anyhow::Error> {
        format::decode(id).map(Into::into)
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", format::encode(&self.0))
    }
}

impl AsRef<OpaqueHash> for PeerId {
    fn as_ref(&self) -> &OpaqueHash {
        &self.0
    }
}

impl FromStr for PeerId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::verify(s)
    }
}

impl From<[u8; 32]> for PeerId {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<PeerId> for [u8; 32] {
    fn from(value: PeerId) -> Self {
        value.0
    }
}
