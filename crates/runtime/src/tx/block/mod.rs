//! Block validations

use crate::{
    Storage,
    storage::{Column, MemoryDb, StateStorage, root},
    tx,
};
use anyhow::Result;
use crypto::merkle;
use pvm::Pvm;
use score::{Block, OpaqueHash, state::StateKeyLike};
use std::{collections::HashMap, sync::Arc};

pub mod header;
pub mod history;

/// DEVELOPMENT: process the block with given state storage.
pub fn process<Vm: Pvm>(block: Block, storage: Arc<impl Storage>) -> Result<()> {
    let state = storage.state()?;
    let mut block2 = block.clone();
    let state2 = state.clone();
    let (vresult, sresult) = rayon::join(
        || header::validate(state, &block.header),
        || tx::simulate_with_state::<Vm>(&mut block2, state2, storage.clone()),
    );

    match (vresult, sresult) {
        (Err(e), _) => anyhow::bail!("failed to import block: {e:?}"),
        (_, Err(e)) => anyhow::bail!("failed to import block: {e:?}"),
        (Ok(()), Ok(diff)) => {
            storage.commit(Column::State, diff)?;
            Ok(())
        }
    }
}

/// Process the block with panic catching.
///
/// Wraps `process` to catch any panics and convert them to `anyhow::Result`.
pub fn checked_process<Vm: Pvm>(block: Block, storage: Arc<impl Storage>) -> Result<()> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        process::<Vm>(block, storage)
    }))
    .unwrap_or_else(|panic| {
        let msg = panic
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".into());
        Err(anyhow::anyhow!("panic during block processing: {msg}"))
    })
}

/// DEVELOPMENT: A test chain for processing fuzz blocks.
pub struct TestChain {
    /// The finalized head of the chain.
    pub finalized: OpaqueHash,

    /// The data of the chain.
    pub data: Arc<MemoryDb>,

    /// The forks and their states.
    pub forks: HashMap<OpaqueHash, Arc<MemoryDb>>,
}

impl TestChain {
    /// Check if the chain is initialized.
    pub fn initialized(&self) -> bool {
        self.finalized != [0; 32]
    }

    /// Import a new block to the chain.
    pub fn import<Vm: Pvm>(&mut self, block: Block) -> anyhow::Result<OpaqueHash> {
        let head = block.header.hash();
        let parent = block.header.parent;

        // Build the guard from the parent fork or finalized state
        let finalize_parent = if let Some(fork) = self.forks.get(&parent) {
            let guard = Arc::new(fork.dup());
            // process the block
            self::checked_process::<Vm>(block, guard.clone())?;
            self.forks.insert(head, guard);
            true
        } else {
            let guard = Arc::new(self.data.dup());
            self::checked_process::<Vm>(block, guard.clone())?;
            self.forks.insert(head, guard);
            false
        };

        if finalize_parent {
            let parent_db = self.forks.remove(&parent).unwrap();
            self.finalized = parent;
            self.data = parent_db;
            // Remove all forks except the one we just inserted
            self.forks.retain(|k, _| *k == head);
        }

        self.forks.get(&head).unwrap().with_data(|data| {
            handle_root(head, data)
        })
    }

    /// Prepare the chain for the given block.
    pub fn prepare(&self, block: &Block) -> (Arc<MemoryDb>, bool) {
        let parent = block.header.parent;

        if let Some(fork) = self.forks.get(&parent) {
            (Arc::new(fork.dup()), true)
        } else {
            (Arc::new(self.data.dup()), false)
        }
    }

    /// Apply the block to the chain.
    pub fn apply(
        &mut self,
        block: &Block,
        guard: Arc<MemoryDb>,
        has_parent_fork: bool,
    ) {
        let parent = block.header.parent;
        let head = block.header.hash();
        self.forks.insert(head, guard);

        if has_parent_fork {
            let parent_db = self.forks.remove(&parent).unwrap();
            self.finalized = parent;
            self.data = parent_db;
            self.forks.retain(|k, _| *k == head);
        }
    }

    /// Initialize the chain with the given block.
    pub fn init(&mut self, state: HashMap<Vec<u8>, Vec<u8>>) -> anyhow::Result<OpaqueHash> {
        self.data.reset(state);
        let head = self
            .data
            .recent_blocks()?
            .last()
            .ok_or(anyhow::anyhow!("no recent blocks"))?
            .header_hash;
        self.finalized = head;
        self.data.with_data(|data| handle_root(head, data))
    }
}

impl Default for TestChain {
    fn default() -> Self {
        Self {
            finalized: Default::default(),
            data: Arc::new(MemoryDb::default()),
            forks: HashMap::new(),
        }
    }
}

/// Compute the state root and cache it for the given header hash.
fn handle_root(head: OpaqueHash, data: &HashMap<Vec<u8>, Vec<u8>>) -> OpaqueHash {
    let mut kvs: Vec<([u8; 31], &[u8])> = data
        .iter()
        .map(|(k, v)| (k.as_slice().as_state_key(), v.as_slice()))
        .collect();
    kvs.sort_by(|a, b| a.0.cmp(&b.0));
    let state_root = merkle::trie31(&kvs);
    root::set(head, state_root);
    state_root
}
