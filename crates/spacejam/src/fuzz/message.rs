//! Fuzz messages

use anyhow::Context;
use score::{Block, OpaqueHash, TimeSlot, TrieKey, block::Header};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, str::FromStr};

/// Messages used in the unix socket communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum Message {
    /// The peer information
    #[serde(rename = "peer-info")]
    Info(PeerInfo) = 0,

    /// The set state
    #[serde(rename = "initialize")]
    Initialize(Initialize) = 1,

    /// The root of the state
    #[serde(rename = "state-root")]
    StateRoot(OpaqueHash) = 2,

    /// The block data
    #[serde(rename = "import-block")]
    ImportBlock(Block) = 3,

    /// The get state
    #[serde(rename = "get-state")]
    GetState(OpaqueHash) = 4,

    /// The state of the peer
    #[serde(rename = "state")]
    State(Vec<KeyValue>) = 5,

    /// The error message
    #[serde(rename = "error")]
    Error(String) = 255,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info(_info) => write!(f, "Info"),
            Self::ImportBlock(block) => {
                write!(
                    f,
                    "ImportBlock(slot={}, hash=0x{})",
                    block.header.slot,
                    hex::encode(block.header.hash())
                )
            }
            Self::Initialize(state) => write!(f, "Initialize(len={})", state.state.len()),
            Self::GetState(hash) => write!(f, "GetState(hash=0x{})", hex::encode(hash)),
            Self::State(state) => write!(f, "State(len={})", state.len()),
            Self::StateRoot(root) => write!(f, "StateRoot(0x{})", hex::encode(root)),
            Self::Error(error) => write!(f, "Error({})", error),
        }
    }
}

/// The peer information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerInfo {
    /// The version of fuzzing
    pub fuzz_version: u8,

    /// The features of fuzzing
    pub fuzz_features: u32,

    /// The version of the peer
    pub jam_version: Version,

    /// The protocol version of the peer
    pub app_version: Version,

    /// The name of the peer
    pub app_name: String,
}

impl Default for PeerInfo {
    fn default() -> Self {
        Self {
            fuzz_version: 1,
            // feature-ancestry (1) | feature-fork (2) — both [M1] mandatory
            fuzz_features: 3,
            jam_version: Version::protocol(),
            app_version: Version::spacejam(),
            app_name: "spacejam".to_string(),
        }
    }
}

/// The version of the peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Version {
    /// The version of the peer
    pub major: u8,

    /// The minor version of the peer
    pub minor: u8,

    /// The patch version of the peer
    pub patch: u8,
}

impl Version {
    /// Binary version, derived from `CARGO_PKG_VERSION_*` at compile time.
    pub fn spacejam() -> Version {
        Version {
            major: env!("CARGO_PKG_VERSION_MAJOR")
                .parse()
                .expect("CARGO_PKG_VERSION_MAJOR not a u8"),
            minor: env!("CARGO_PKG_VERSION_MINOR")
                .parse()
                .expect("CARGO_PKG_VERSION_MINOR not a u8"),
            patch: env!("CARGO_PKG_VERSION_PATCH")
                .parse()
                .expect("CARGO_PKG_VERSION_PATCH not a u8"),
        }
    }

    /// JAM protocol version, sourced from `[workspace.metadata.graypaper]`
    /// in the workspace manifest via the build script.
    pub fn protocol() -> Version {
        env!("GRAYPAPER_VERSION")
            .parse()
            .expect("invalid graypaper version")
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');
        let major = parts.next().context("missing major")?.parse()?;
        let minor = parts.next().context("missing minor")?.parse()?;
        let patch = parts.next().context("missing patch")?.parse()?;
        if parts.next().is_some() {
            anyhow::bail!("version has too many components");
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

/// A key-value pair
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyValue {
    /// The key of the key-value pair
    pub key: TrieKey,

    /// The value of the key-value pair
    pub value: Vec<u8>,
}

/// Set the state of the peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Initialize {
    /// The header of the block
    pub header: Header,

    /// The state of the peer
    pub state: Vec<KeyValue>,

    /// The ancestry of the peer
    pub ancestry: Vec<Head>,
}

impl Initialize {
    /// Get the key-value pairs of the state
    pub fn keyvals(&self) -> HashMap<Vec<u8>, Vec<u8>> {
        self.state
            .iter()
            .map(|kv| (kv.key.to_vec(), kv.value.clone()))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Head {
    /// The slot of the head
    #[serde(rename = "slot")]
    pub slot: TimeSlot,

    /// The hash of the head
    #[serde(rename = "header-hash")]
    pub hash: OpaqueHash,
}
