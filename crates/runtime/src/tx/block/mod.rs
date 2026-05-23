//! Block validations

use crate::{
    Storage,
    storage::{Branch, Column, Commit, KVStorage, MemoryDb, MultiTree, StateStorage, root},
    tx,
};
use anyhow::Result;
use pvm::Pvm;
use score::{Block, OpaqueHash, TrieKey, state::StateKeyLike};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

pub mod header;
pub mod history;

/// Zero hash sentinel — `current_root` before [`TestChain::init`].
const EMPTY_ROOT: OpaqueHash = [0; 32];

type Fork = Branch<MemoryDb>;

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
        (Err(e), _) | (_, Err(e)) => Err(e),
        (Ok(()), Ok(diff)) => {
            storage.commit(Column::State, &diff)?;
            Ok(())
        }
    }
}

/// DEVELOPMENT: A test chain for processing fuzz blocks.
pub struct TestChain {
    /// The finalized head of the chain.
    pub finalized: OpaqueHash,

    /// The data of the chain.
    pub data: Arc<MemoryDb>,

    /// The forks and their states (diff overlay + the state root computed
    /// at import time via [`MultiTree::apply`]).
    pub forks: HashMap<OpaqueHash, (Fork, OpaqueHash)>,

    /// The state root corresponding to `data`. Tracked incrementally via the
    /// multitree column so we can skip the O(N log N) full retrie per block.
    current_root: OpaqueHash,
}

impl TestChain {
    /// Check if the chain is initialized.
    pub fn initialized(&self) -> bool {
        self.finalized != [0; 32]
    }

    /// Compute and persist the post-block state root incrementally.
    fn compute_fork_root(&self, commit: &Commit<TrieKey, Vec<u8>>) -> Result<OpaqueHash> {
        let prev = (self.current_root != EMPTY_ROOT).then_some(self.current_root);
        let dirty = commit.dirty_keys();
        self.data
            .with_data(|base| self.data.apply(prev, &commit.merge_with(base), &dirty))?
    }

    /// Commit a fork's diff into `self.data` and release orphan siblings.
    fn finalize_fork(&mut self, parent: OpaqueHash) -> Result<()> {
        if let Some((fork, parent_root)) = self.forks.remove(&parent) {
            let commit = fork
                .commit
                .read()
                .map_err(|_| anyhow::anyhow!("lock poisoned"))?
                .clone();
            self.data.commit(Column::State, &commit)?;

            // Release sibling trees that won't be finalized.
            for (_, sibling_root) in self.forks.values() {
                self.data.dereference_tree(*sibling_root)?;
            }

            self.finalized = parent;
            self.current_root = parent_root;
            self.forks.clear();
        }
        Ok(())
    }

    /// Import a new block to the chain.
    pub fn import<Vm: Pvm>(&mut self, block: Block) -> Result<OpaqueHash> {
        let head = block.header.hash();
        let parent = block.header.parent;
        if self.forks.contains_key(&parent) {
            self.finalize_fork(parent)?;
        }

        // Process on a Branch overlay
        let guard = Arc::new(Branch::checkout(self.data.clone()));
        self::process::<Vm>(block, guard.clone())?;

        // Compute the post-block root incrementally from the overlay diff.
        let state_root = {
            let commit = guard
                .commit
                .read()
                .map_err(|_| anyhow::anyhow!("lock poisoned"))?;
            self.compute_fork_root(&commit)?
        };
        root::set(head, state_root);

        // Store Branch (diff only) + its root in forks
        let fork = Arc::try_unwrap(guard).unwrap_or_else(|arc| (*arc).clone());
        self.forks.insert(head, (fork, state_root));
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
    pub fn apply(&mut self, block: &Block, guard: Arc<Fork>) -> Result<()> {
        let head = block.header.hash();
        let state_root = {
            let commit = guard
                .commit
                .read()
                .map_err(|_| anyhow::anyhow!("lock poisoned"))?;
            self.compute_fork_root(&commit)?
        };
        root::set(head, state_root);
        let fork = Arc::try_unwrap(guard).unwrap_or_else(|arc| (*arc).clone());
        self.forks.insert(head, (fork, state_root));
        Ok(())
    }

    /// Initialize the chain with the given block.
    pub fn init(&mut self, state: HashMap<Vec<u8>, Vec<u8>>) -> anyhow::Result<OpaqueHash> {
        let state: BTreeMap<TrieKey, Vec<u8>> = state
            .into_iter()
            .map(|(k, v)| (k.as_slice().as_state_key(), v))
            .collect();
        self.data.reset(state);
        let head = self
            .data
            .recent_blocks()?
            .last()
            .ok_or(anyhow::anyhow!("no recent blocks"))?
            .header_hash;
        self.finalized = head;

        let new_root = self.data.with_data(|data| {
            let kvs: Vec<(TrieKey, &[u8])> = data.iter().map(|(k, v)| (*k, v.as_slice())).collect();
            let dirty: Vec<TrieKey> = data.keys().copied().collect();
            self.data.apply(None, &kvs, &dirty)
        })??;
        root::set(head, new_root);
        self.current_root = new_root;
        Ok(new_root)
    }
}

impl Default for TestChain {
    fn default() -> Self {
        Self {
            finalized: Default::default(),
            data: Arc::new(MemoryDb::default()),
            forks: HashMap::new(),
            current_root: EMPTY_ROOT,
        }
    }
}
