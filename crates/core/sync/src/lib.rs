//! Block sync validation

use anyhow::Result;
use score::{block::History, Block, State};

pub mod assurance;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
pub mod statistic;
pub mod ticket;

/// Block sync validation result
pub struct Sync;

/// Validate new block and transit to the next state
///
/// TODO: do state transition following the dependency graph: GP 4.2.1
pub fn transit(block: &Block, state: &State, entropy: [u8; 32]) -> Result<Sync> {
    let mut next = state.clone();

    // (4.5) τ' ≺ H
    //
    // The new timeslot index (τ') depends directly on the block header (H)
    next.timeslot = block.header.slot;
    Ok(Sync)
}
