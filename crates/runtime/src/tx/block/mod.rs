//! Block validations

use crate::{
    Storage,
    storage::{Column, MemoryDb},
    tx,
};
use anyhow::Result;
use pvm::Pvm;
use score::{Block, OpaqueHash};
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
    /// Initialize the chain with the given block.
    pub fn new<Vm: Pvm>(block: Block) -> anyhow::Result<Self> {
        let head = block.header.hash();
        let data = Arc::new(MemoryDb::default());
        let mut forks = HashMap::new();

        // import the genesis block
        self::process::<Vm>(block, data.clone())?;
        forks.insert(head, data.deep_clone());
        Ok(Self {
            finalized: head,
            data,
            forks,
        })
    }

    /// Import a new block to the chain.
    pub fn import<Vm: Pvm>(&mut self, block: Block) -> anyhow::Result<()> {
        let head = block.header.hash();
        let parent = block.header.parent;
        let guard = Arc::new(self.data.dup());
        let mut pstate = None;

        // find the parent of the block
        if let Some(state) = self.forks.get(&parent) {
            pstate = Some(state.clone());
            guard.reset(state.clone());
        }

        // process the block
        self::process::<Vm>(block, guard.clone())?;
        if let Some(pstate) = pstate {
            self.finalized = parent;
            self.data.reset(pstate);
            self.forks.clear();
        }

        // update the forks
        self.forks.insert(head, guard.deep_clone());
        Ok(())
    }
}
