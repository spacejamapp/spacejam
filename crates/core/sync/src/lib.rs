//! Block sync validation

use anyhow::Result;
use score::{Block, State};

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

    let (_tickets, _epoch) = ticket::validate(&mut next, block, entropy)?;
    // next.recent_blocks.push(block.header.clone());

    Ok(Sync)
}
