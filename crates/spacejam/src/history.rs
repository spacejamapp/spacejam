use score::{
    block::history::{BlockInfo, BlocksHistory, Mmr, ReportedWorkPackage},
    misc::OpaqueHash,
};

/// A chain of blocks.
#[derive(Debug, Clone)]
pub struct History(pub BlocksHistory);

impl History {
    /// Import a new block into the chain according to graypaper section 7.1-7.4.
    pub fn import(
        &mut self,
        header_hash: OpaqueHash,
        state_root: OpaqueHash,
        accumulated_root: OpaqueHash,
        reported: Vec<ReportedWorkPackage>,
    ) {
        let Some(last) = self.0.blocks.last_mut() else {
            self.0.blocks.push(BlockInfo {
                header_hash,
                mmr: Mmr {
                    peaks: vec![Some(accumulated_root)],
                },
                state_root: OpaqueHash::default(),
                reported,
            });
            return;
        };

        // Update the state root of the parent block if it exists (formula 7.2)
        // β† ≡ β exc β†[|β| - 1]_s = H_r
        last.state_root = state_root;
        let mut mmr = last.mmr.clone();
        mmr.append(accumulated_root);

        // Create new block info according to formula 7.3:
        // let n = (p, h: H(H), b, s: H^0)
        // where:
        // - p is the work reports map from reported packages
        // - h is the header hash
        // - b is the MMR with accumulated root appended
        // - s is initialized to zero state root
        let new_block = BlockInfo {
            header_hash,
            state_root: OpaqueHash::default(), // Initialize to zero/default
            mmr,
            reported,
        };

        // Append the new block to history
        self.0.blocks.push(new_block);

        // Truncate to maintain history size limit
        if self.0.blocks.len() > score::MAX_BLOCKS_HISTORY {
            self.0.blocks.remove(0);
        }
    }
}
