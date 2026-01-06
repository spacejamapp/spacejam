//! Block validations

use crate::{Storage, storage::Column, tx};
use anyhow::Result;
use pvm::Pvm;
use score::Block;
use std::sync::Arc;

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
