//! Authorization for the guarantees extrinsic.

use anyhow::Result;
use score::{Block, State};

/// Handle the guarantees extrinsic.
///
/// TODO: check indices
pub fn validate(state: &mut State, block: &Block) -> Result<()> {
    let slot = block.header.slot;
    let guarantees = &block.extrinsic.guarantees;

    // Process each guarantee
    let mut processed = Vec::new();
    for guarantee in guarantees {
        // Consume the authorizer from the pool
        state.pools[guarantee.report.core_index as usize] = state.pools
            [guarantee.report.core_index as usize]
            .iter()
            .filter(|pool| **pool != guarantee.report.authorizer_hash)
            .cloned()
            .collect();

        // mark the core as processed
        processed.push(guarantee.report.core_index as usize);
    }

    // add new authorizers from queue to the pools
    for (core_index, pool) in state.pools.iter_mut().enumerate() {
        if !processed.contains(&core_index) {
            *pool = pool[1..].into();
        }

        pool.push(state.authorization[core_index][slot as usize]);
    }

    Ok(())
}
