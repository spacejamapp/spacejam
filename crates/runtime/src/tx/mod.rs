//! Block sync validation

use crate::{
    Storage,
    storage::{Column, Commit},
    timing,
};
use anyhow::Result;
pub use executor::Executor;
use pvm::Pvm;
use score::{Block, TrieKey};
use std::sync::Arc;

pub mod assurance;
pub mod block;
pub mod dispute;
pub mod executor;
pub mod guarantee;
pub mod preimage;
pub mod ticket;

/// Transit state with new block
#[tracing::instrument(skip_all, name = "stf")]
pub fn transit<Vm: Pvm>(
    mut block: Block,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let diff = self::simulate::<Vm>(&mut block, storage.clone())?;
    let _guard = timing::commit();
    storage.commit(Column::State, &diff)?;
    Ok(diff)
}

/// Transit state with new block
#[tracing::instrument(skip_all, name = "stf")]
pub fn transit_with_state<Vm: Pvm>(
    mut block: Block,
    state: score::State,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let diff = self::simulate_with_state::<Vm>(&mut block, state, storage.clone())?;
    let _guard = timing::commit();
    storage.commit(Column::State, &diff)?;
    Ok(diff)
}

/// Simulate state transition with new block
pub fn simulate<Vm: Pvm>(
    block: &mut Block,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let state = storage.state()?;
    self::simulate_with_state::<Vm>(block, state, storage.clone())
}

/// Simulate state transition with new block
pub fn simulate_with_state<Vm: Pvm>(
    block: &mut Block,
    state: score::State,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    Executor::<Vm, _>::new(block, state, storage).run()
}
