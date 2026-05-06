//! Fuzz messages

use score::{block::Header, Block, OpaqueHash, TimeSlot, TrieKey};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

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
            fuzz_features: 2,
            jam_version: Version::PROTOCOL,
            app_version: Version::SPACEJAM,
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
    /// The binary version of spacejam
    pub const SPACEJAM: Version = Version {
        major: 0,
        minor: 1,
        patch: 1,
    };

    /// The protocol version of spacejam
    pub const PROTOCOL: Version = Version {
        major: 0,
        minor: 7,
        patch: 2,
    };
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
