//! Block history utilities

use crypto::merkle::mmr;
use score::{
    OpaqueHash,
    block::{BlockInfo, History},
    service::ReportedWorkPackage,
};

/// Import a new block into the history
pub fn import(
    history: &mut History,
    header_hash: OpaqueHash,
    accumulate_root: OpaqueHash,
    reported: Vec<ReportedWorkPackage>,
) {
    history.mmr.peaks = mmr::append(history.mmr.peaks.clone(), accumulate_root);
    if history.history.is_empty() {
        let new_block = BlockInfo {
            header_hash,
            beefy_root: accumulate_root,
            state_root: OpaqueHash::default(),
            reported,
        };

        history.history.push(new_block);
        return;
    };

    // compose block info
    let beefy_root = mmr::root(&history.mmr.peaks).unwrap_or_default();
    let new_block = BlockInfo {
        header_hash,
        beefy_root,
        state_root: OpaqueHash::default(),
        reported,
    };
    history.history.push(new_block);

    // Truncate to maintain history size limit
    if history.history.len() > score::MAX_BLOCKS_HISTORY as usize {
        history.history.remove(0);
    }
}
