//! Debug utilities for the state root.

use crate::{
    chain::Fork,
    storage::{Archive, StateStorage, SyncStorage},
    Storage,
};
use score::{block::BlockJson, Block, OpaqueHash, TimeSlot};
use spacejson::Json;
use std::{collections::BTreeMap, fs};

impl<S: Storage> Fork<S> {
    /// capture the state on state root mismatch
    pub fn on_state_root_mismatch(
        &self,
        block: Block,
        expected: OpaqueHash,
        got: OpaqueHash,
    ) -> anyhow::Result<()> {
        let mut parent = block.header.parent;
        let mut message = StateRootMismatch {
            slot: block.header.slot,
            expected,
            got,
            chain: vec![],
        };

        for _ in 0..12 {
            let archive = Archive::checkout(self.state.clone(), parent);
            let root = archive.root()?;
            let mut state = BTreeMap::new();
            let iter = archive.state_iter()?;
            for pair in iter {
                let (key, value) = pair?;
                state.insert(hex::encode(key), hex::encode(value));
            }
            message.chain.push(BlockWithState {
                block: block.clone(),
                state,
                root,
            });

            parent = self.state.parent(&parent)?;
        }

        let message = serde_json::to_string(&message.to_json())?;
        println!("{message}");
        fs::write("state_root_mismatch.json", message)?;
        Ok(())
    }
}

/// A struct representing a state root mismatch.
#[derive(Debug, Clone, Json)]
pub struct StateRootMismatch {
    /// The slot of the block.
    pub slot: TimeSlot,

    /// The expected state root.
    #[json(hex)]
    pub expected: OpaqueHash,

    /// The got state root.
    #[json(hex)]
    pub got: OpaqueHash,

    /// The chain of blocks that caused the state root mismatch.
    #[json(Vec<BlockWithStateJson>)]
    pub chain: Vec<BlockWithState>,
}

/// A struct representing a block with its state.
#[derive(Debug, Clone, Json)]
pub struct BlockWithState {
    /// The block.
    #[json(nested)]
    pub block: Block,

    /// The state.
    pub state: BTreeMap<String, String>,

    /// The state root.
    #[json(hex)]
    pub root: OpaqueHash,
}
