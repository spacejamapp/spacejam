//! Debug utilities for the state root.

use crate::{
    chain::Fork,
    storage::{Archive, StateStorage, SyncStorage},
    Storage,
};
use score::{Block, OpaqueHash};
use serde_json::json;
use spacejson::Json;
use std::{env, fs, path::Path};

const TRACES_OUTPUT: &str = "TRACES_OUTPUT";

impl<S: Storage> Fork<S> {
    /// Extract state from a block hash
    fn extract_state(&self, block_hash: OpaqueHash) -> anyhow::Result<State> {
        let archive = Archive::checkout(self.state.clone(), block_hash);
        let root = archive.root()?;
        let mut keyvals = Vec::new();
        let iter = archive.state_iter()?;
        for pair in iter {
            let (key, value) = pair?;
            keyvals.push(KeyValue {
                key: format!("0x{}", hex::encode(key)),
                value: format!("0x{}", hex::encode(value)),
            });
        }
        Ok(State {
            state_root: format!("0x{}", hex::encode(root)),
            keyvals,
        })
    }

    /// capture the state on state root mismatch
    pub fn on_state_root_mismatch(
        &self,
        block: Block,
        _expected: OpaqueHash,
        _got: OpaqueHash,
    ) -> anyhow::Result<()> {
        let mut parent = block.header.parent;

        // Create output directory if it doesn't exist
        let dir = env::var(TRACES_OUTPUT).unwrap_or_else(|_| "traces".to_string());
        let output_dir = Path::new(&dir);
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }

        // Get the initial pre_state (parent block state)
        let mut pre_state = self.extract_state(parent)?;

        // Generate 12 separate files for recent blocks
        for i in 0..12 {
            // Move to the next block (parent becomes the current block)
            parent = self.state.parent(&parent)?;

            // Get post_state (current block state)
            let post_state = self.extract_state(parent)?;

            let filename = format!("{:08}.json", i + 1);
            let filepath = output_dir.join(filename);
            let json_content = json!({
                "pre_state": pre_state,
                "block": block.clone().to_json(),
                "post_state": post_state,
            })
            .to_string();
            fs::write(filepath, json_content)?;

            // Update pre_state for next iteration
            pre_state = post_state;
        }

        println!("Generated 12 trace files in {}", TRACES_OUTPUT);
        Ok(())
    }
}

/// A struct representing a key-value pair in the state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyValue {
    /// The key in hex format.
    pub key: String,

    /// The value in hex format.
    pub value: String,
}

/// A struct representing the pre-state with state root and key-value pairs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
    /// The state root in hex format.
    pub state_root: String,

    /// The key-value pairs in the state.
    pub keyvals: Vec<KeyValue>,
}
