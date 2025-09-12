//! Fuzz messages

use std::fmt::Display;

use score::{Block, OpaqueHash, TrieKey, block::Header};
use serde::{Deserialize, Serialize};

/// Messages used in the unix socket communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Message {
    /// The peer information
    #[serde(rename = "peer-info")]
    Info(PeerInfo),

    /// The block data
    #[serde(rename = "import-block")]
    ImportBlock(Block),

    /// The set state
    #[serde(rename = "set-state")]
    SetState(SetState),

    /// The get state
    #[serde(rename = "get-state")]
    GetState(OpaqueHash),

    /// The state of the peer
    #[serde(rename = "state")]
    State(Vec<KeyValue>),

    /// The root of the state
    #[serde(rename = "state-root")]
    StateRoot(OpaqueHash),
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
                    hex::encode(block.header.hash().unwrap())
                )
            }
            Self::SetState(state) => write!(f, "SetState(len={})", state.state.len()),
            Self::GetState(hash) => write!(f, "GetState(hash=0x{})", hex::encode(hash)),
            Self::State(state) => write!(f, "State(len={})", state.len()),
            Self::StateRoot(root) => write!(f, "StateRoot(0x{})", hex::encode(root)),
        }
    }
}

/// The peer information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerInfo {
    /// The name of the peer
    pub name: String,

    /// The version of the peer
    pub version: Version,

    /// The protocol version of the peer
    pub protocol: Version,
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
pub struct SetState {
    /// The header of the block
    pub header: Header,

    /// The state of the peer
    pub state: Vec<KeyValue>,
}
