//! Block validations

use crate::{
    Storage,
    storage::{Branch, Column, KVStorage, MemoryDb, StateStorage, root},
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

type Fork = Branch<MemoryDb>;

/// DEVELOPMENT: A test chain for processing fuzz blocks.
pub struct TestChain {
    /// The finalized head of the chain.
    pub finalized: OpaqueHash,

    /// The data of the chain.
    pub data: Arc<MemoryDb>,

    /// The forks and their states (diff only).
    pub forks: HashMap<OpaqueHash, Fork>,
}

impl TestChain {
    /// Check if the chain is initialized.
    pub fn initialized(&self) -> bool {
        self.finalized != [0; 32]
    }

    /// Finalize a parent fork by committing its diff into self.data.
    fn finalize_fork(&mut self, parent: OpaqueHash) -> anyhow::Result<()> {
        if let Some(fork) = self.forks.remove(&parent) {
            let commit = fork
                .commit
                .read()
                .map_err(|_| anyhow::anyhow!("lock poisoned"))?
                .clone();
            self.data.commit(Column::State, commit)?;
            self.finalized = parent;
            self.forks.clear();
        }
        Ok(())
    }

    /// Import a new block to the chain.
    pub fn import<Vm: Pvm>(&mut self, block: Block) -> anyhow::Result<OpaqueHash> {
        let head = block.header.hash();
        let parent = block.header.parent;

        if self.forks.contains_key(&parent) {
            self.finalize_fork(parent)?;
        }

        // Process on a Branch overlay
        let guard = Arc::new(Branch::checkout(self.data.clone()));
        self::process::<Vm>(block, guard.clone())?;
        let state_root = guard.root()?;
        root::set(head, state_root);

        // Store Branch (diff only) in forks
        self.forks.insert(
            head,
            Arc::try_unwrap(guard).unwrap_or_else(|arc| (*arc).clone()),
        );
        Ok(state_root)
    }

    /// Prepare the chain for the given block.
    pub fn prepare(&mut self, block: &Block) -> Arc<Fork> {
        let parent = block.header.parent;
        if self.forks.contains_key(&parent) {
            let _ = self.finalize_fork(parent);
        }
        Arc::new(Branch::checkout(self.data.clone()))
    }

    /// Apply the block to the chain.
    pub fn apply(&mut self, block: &Block, guard: Arc<Fork>) {
        let head = block.header.hash();
        self.forks.insert(
            head,
            Arc::try_unwrap(guard).unwrap_or_else(|arc| (*arc).clone()),
        );
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
