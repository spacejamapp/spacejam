use score::{
    block::history::{BlocksHistory, ReportedWorkPackage},
    misc::OpaqueHash,
};

/// A chain of blocks.
#[derive(Debug, Clone)]
pub struct History(pub BlocksHistory);

impl History {
    /// Import a new block into the chain according to graypaper section 7.1-7.4.
    pub fn import(
        &mut self,
        _header_hash: OpaqueHash,
        state_root: OpaqueHash,
        _accumulated_root: OpaqueHash,
        _reported: Vec<ReportedWorkPackage>,
    ) {
        // Update the state root of the parent block if it exists (formula 7.2)
        // β† ≡ β exc β†[|β| - 1]_s = H_r
        if let Some(last) = self.0.blocks.last_mut() {
            last.state_root = state_root;
        }

        // Create new block info with:
        // - Header hash
        // - Accumulation result MMR
        // - Work package hashes
        // Then append to history while maintaining size limit H (formula 7.3)
        // β' ≡ overleftarrow{β† ++ n}^H

        // TODO:

        // Truncate to maintain history size limit
        if self.0.blocks.len() > score::MAX_BLOCKS_HISTORY {
            self.0.blocks.remove(0);
        }
    }
}
