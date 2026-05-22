//! Block validations

use crate::{
    Storage,
    storage::{Branch, Column, Commit, KVStorage, MemoryDb, MultiTreeStore, StateStorage, root},
    tx,
};
use anyhow::Result;
use crypto::merkle;
use pvm::Pvm;
use score::{Block, OpaqueHash, TrieKey, state::StateKeyLike};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

/// Zero hash sentinel — `current_root` before [`TestChain::init`].
const EMPTY_ROOT: OpaqueHash = [0; 32];

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
        (Err(e), _) | (_, Err(e)) => Err(e),
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

    /// The state root corresponding to `data`. Tracked incrementally via the
    /// multitree column so we can skip the O(N log N) full retrie per block.
    current_root: OpaqueHash,
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
            let dirty = collect_dirty_keys(&commit);
            self.data.commit(Column::State, commit)?;

            // Incrementally rebuild the trie against the new `data` and prime
            // the root cache so the next block's `state()` hits it.
            let prev = (self.current_root != EMPTY_ROOT).then_some(self.current_root);
            let new_root = self.data.with_data(|data| {
                let kvs: Vec<(TrieKey, &[u8])> =
                    data.iter().map(|(k, v)| (*k, v.as_slice())).collect();
                self.data.apply(Column::TrieNodes, prev, &kvs, &dirty)
            })??;
            root::set(parent, new_root);
            self.current_root = new_root;

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

        // Compute root from base HashMap + overlay diff (no clone of base)
        let state_root = {
            let commit = guard
                .commit
                .read()
                .map_err(|_| anyhow::anyhow!("lock poisoned"))?;
            self.data
                .with_data(|data| handle_root_with_diff(head, data, &commit))?
        };

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

        // Bootstrap the incremental trie. Every key is dirty on the first
        // build; the algorithm degenerates to a full retrie.
        let new_root = self.data.with_data(|data| {
            let kvs: Vec<(TrieKey, &[u8])> = data.iter().map(|(k, v)| (*k, v.as_slice())).collect();
            let dirty: Vec<TrieKey> = data.keys().copied().collect();
            self.data.apply(Column::TrieNodes, None, &kvs, &dirty)
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

/// Sorted, deduplicated union of the keys an overlay commit touches.
fn collect_dirty_keys(commit: &Commit<TrieKey, Vec<u8>>) -> Vec<TrieKey> {
    let mut keys: Vec<TrieKey> = commit
        .update
        .keys()
        .copied()
        .chain(commit.removal.iter().copied())
        .collect();
    keys.sort_unstable();
    keys.dedup();
    keys
}

/// Compute the state root from base data + overlay diff (no base clone).
///
/// Both base and diff.update are sorted by TrieKey, so we merge-walk them in
/// O(N+M) and skip the final sort. On ties diff wins (it's the newer write);
/// keys present in diff.removal are dropped after selection so removals take
/// precedence over updates of the same key.
///
/// NOTE: this method overrides the StateStorage::root for zero-copy.
fn handle_root_with_diff(
    head: OpaqueHash,
    base: &BTreeMap<TrieKey, Vec<u8>>,
    diff: &crate::storage::Commit<TrieKey, Vec<u8>>,
) -> OpaqueHash {
    let mut kvs: Vec<(TrieKey, &[u8])> = Vec::with_capacity(base.len() + diff.update.len());
    let mut base_iter = base.iter();
    let mut diff_iter = diff.update.iter();
    let mut b = base_iter.next();
    let mut d = diff_iter.next();

    while let Some((key, value)) = match (b, d) {
        (Some((bk, _)), Some((dk, dv))) if dk <= bk => {
            if dk == bk {
                b = base_iter.next();
            }
            d = diff_iter.next();
            Some((dk, dv.as_slice()))
        }
        (Some((bk, bv)), _) => {
            b = base_iter.next();
            Some((bk, bv.as_slice()))
        }
        (None, Some((dk, dv))) => {
            d = diff_iter.next();
            Some((dk, dv.as_slice()))
        }
        (None, None) => None,
    } {
        if !diff.removal.contains(key) {
            kvs.push((*key, value));
        }
    }

    let state_root = merkle::trie31(&kvs);
    root::set(head, state_root);
    state_root
}
