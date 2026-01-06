//! Block validations

/* use crate::{Storage, storage::Commit, tx};
use anyhow::Result;
use pvm::Pvm;
use score::{Block, State, TrieKey};
use std::sync::Arc; */

pub mod header;
pub mod history;
// mod processor;

/* pub fn process<Vm: Pvm>(
    block: Block,
    storage: Arc<impl Storage>,
) -> Result<Commit<TrieKey, Vec<u8>>> {
    let state = storage.state()?;
    let mut block2 = block.clone();
    let state2 = state.clone();
    let (vresult, sresult) = rayon::join(
        || header::validate(state, &block.header),
        || tx::simulate_with_state::<Vm>(&mut block2, state2, storage.clone()),
    );
}
 */
