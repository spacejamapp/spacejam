//! Peer related stuffs

pub use address::Address;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

mod address;

/// Peer ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(String);

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for PeerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for PeerId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.len();
        if len != 53 {
            anyhow::bail!("invalid peer id length, should be 53, got {len}");
        }

        if !s.starts_with("e") {
            anyhow::bail!(
                "invalid peer id prefix, should be 'e', got '{:?}'",
                s.chars().next()
            );
        }

        // Check if the peer id is valid base32
        let base32 = s.split_at(2).1;
        let _ = base32::decode(base32::Alphabet::Rfc4648Lower { padding: false }, base32)
            .ok_or_else(|| anyhow::anyhow!("peer id is not valid base32"))?;

        Ok(Self(s.to_string()))
    }
}

impl From<&[u8; 32]> for PeerId {
    fn from(value: &[u8; 32]) -> Self {
        Self(format!(
            "e{}",
            base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, value)
        ))
    }
}

impl TryFrom<PeerId> for [u8; 32] {
    type Error = anyhow::Error;

    fn try_from(value: PeerId) -> Result<Self, Self::Error> {
        let base32 = value.0.split_at(2).1;
        let bytes = base32::decode(base32::Alphabet::Rfc4648Lower { padding: false }, base32)
            .ok_or_else(|| anyhow::anyhow!("failed to decode peer id"))?;

        bytes
            .try_into()
            .map_err(|e| anyhow::anyhow!("failed to convert peer id bytes to [u8; 32]: {e:?}"))
    }
}
