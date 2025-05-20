//! Chain Specifications

use crate::chain::ChainId;
use anyhow::Result;
use network::Address;
use score::block::Header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Development chain spec
const DEV_SPEC: &str = include_str!("../../spec/dev/spec.json");

/// Chain Specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    /// List of bootnodes
    pub bootnodes: Vec<String>,

    /// The chain id
    pub id: String,

    /// genesis state kv map
    pub genesis_state: HashMap<String, String>,

    /// genesis header in hex format
    pub genesis_header: String,
}

impl Spec {
    /// Create a new dev chain spec
    pub fn dev() -> Self {
        serde_json::from_str(DEV_SPEC).expect("failed to parse dev spec")
    }

    /// Parse the spec
    pub fn parse(self) -> Result<ParsedSpec> {
        let genesis_header: Header = codec::decode(&hex::decode(&self.genesis_header)?)?;
        let genesis_state: HashMap<[u8; 31], Vec<u8>> = self
            .genesis_state
            .into_iter()
            .map(|(k, v)| {
                let mut key = [0u8; 31];
                key.copy_from_slice(&hex::decode(k)?);
                let value = hex::decode(v)?;
                Ok((key, value))
            })
            .collect::<Result<_>>()?;

        Ok(ParsedSpec {
            bootnodes: self
                .bootnodes
                .iter()
                .map(|s| s.parse())
                .collect::<Result<Vec<_>>>()?,
            id: self.id.parse()?,
            genesis_state,
            genesis_header,
        })
    }
}

/// Parsed chain spec
pub struct ParsedSpec {
    /// List of bootnodes
    pub bootnodes: Vec<Address>,

    /// The chain id
    pub id: ChainId,

    /// genesis state kv map
    pub genesis_state: HashMap<[u8; 31], Vec<u8>>,

    /// genesis header
    pub genesis_header: Header,
}
