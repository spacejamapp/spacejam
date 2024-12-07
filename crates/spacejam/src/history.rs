use mmr::{util::MemStore, Merge, Result, MMR};
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
        // Update the state root of the parent block if it exists (formula 7.2)
        // β† ≡ β exc β†[|β| - 1]_s = H_r
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

        last.state_root = state_root;
        // Create new block info according to formula 7.3:
        // let n = (p, h: H(H), b, s: H^0)
        // where:
        // - p is the work reports map from reported packages
        // - h is the header hash
        // - b is the MMR with accumulated root appended
        // - s is initialized to zero state root
        let peaks = last.mmr.peaks.clone();
        let new_block = BlockInfo {
            header_hash,
            state_root: OpaqueHash::default(), // Initialize to zero/default
            mmr: setup_mmr(self.0.blocks.len() as u8, peaks, accumulated_root),
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

/// Setup the MMR with the previous items and the accumulated root.
fn setup_mmr(
    history_size: u8,
    peaks: Vec<Option<OpaqueHash>>,
    accumulated_root: OpaqueHash,
) -> Mmr {
    let store = MemStore::default();
    let mut mmr = MMR::<_, Keccak, _>::new(0, &store);
    let range = 2u8.pow(peaks.len() as u32) - 1;

    // Push all the preview peaks
    for peak in peaks {
        mmr.push(peak).unwrap();
    }

    // Push the accumulated root
    //
    // TODO: check if we can replace the empty leaves. (use append)
    mmr.push(Some(accumulated_root)).unwrap();

    // Push the missing peaks
    if history_size == range {
        // TODO: append directly to the MMR
        for _ in 0..range {
            mmr.push(None).unwrap();
        }
    }

    // Get the peaks and reverse them
    let mut peaks = mmr.get_ancestor_peaks_and_root(mmr.mmr_size()).unwrap().0;
    peaks.reverse();

    Mmr { peaks }
}

pub struct Keccak;

impl Merge for Keccak {
    type Item = Option<OpaqueHash>;

    fn merge(left: &Self::Item, right: &Self::Item) -> Result<Self::Item> {
        let (Some(left), Some(right)) = (left, right) else {
            // If either input is None, the result is None
            return Ok(None);
        };

        // Concatenate and hash the inputs as per the spec
        let input = [*left, *right].concat();
        Ok(Some(crypto::keccak(&input)))
    }
}
