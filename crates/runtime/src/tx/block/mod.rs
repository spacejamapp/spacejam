//! Block validations

use crate::{
    Storage,
    storage::{Column, MemoryDb, StateStorage},
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
    pub forks: HashMap<OpaqueHash, HashMap<Vec<u8>, Vec<u8>>>,
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

        // Avoid dup() when a fork exists — it would be immediately overwritten by reset()
        let (guard, pstate) = if let Some(state) = self.forks.get(&parent) {
            let cloned = state.clone();
            let guard = Arc::new(MemoryDb::default());
            guard.reset(cloned.clone());
            (guard, Some(cloned))
        } else {
            (Arc::new(self.data.dup()), None)
        };

        // process the block
        self::checked_process::<Vm>(block, guard.clone())?;
        if let Some(pstate) = pstate {
            self.finalized = parent;
            self.data.reset(pstate);
            self.forks.clear();
        }

        // Combine deep_clone + root computation under a single read lock
        guard.with_data(|data| {
            self.forks.insert(head, data.clone());

            let mut kvs: Vec<([u8; 31], &[u8])> = data
                .iter()
                .map(|(k, v)| (k.as_slice().as_state_key(), v.as_slice()))
                .collect();
            kvs.sort_by(|a, b| a.0.cmp(&b.0));
            merkle::trie31(&kvs)
        })
    }

    /// Prepare the chain for the given block.
    #[allow(clippy::type_complexity)]
    pub fn prepare(&self, block: &Block) -> (Arc<MemoryDb>, Option<HashMap<Vec<u8>, Vec<u8>>>) {
        let parent = block.header.parent;

        // Avoid dup() when a fork exists — it would be immediately overwritten by reset()
        if let Some(state) = self.forks.get(&parent) {
            let cloned = state.clone();
            let guard = Arc::new(MemoryDb::default());
            guard.reset(cloned.clone());
            (guard, Some(cloned))
        } else {
            (Arc::new(self.data.dup()), None)
        }
    }

    /// Apply the block to the chain.
    pub fn apply(
        &mut self,
        block: &Block,
        guard: Arc<MemoryDb>,
        pstate: Option<HashMap<Vec<u8>, Vec<u8>>>,
    ) {
        let parent = block.header.parent;
        if let Some(pstate) = pstate {
            self.finalized = parent;
            self.data.reset(pstate);
            self.forks.clear();
        }

        self.forks.insert(block.header.hash(), guard.deep_clone());
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
        self.data.root()
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
