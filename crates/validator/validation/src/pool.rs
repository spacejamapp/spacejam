//! Extrinsic Pool

use async_lock::RwLock;
use score::{block::Extrinsic, consensus::Safrole, misc::OpaqueHash};
use std::{collections::BTreeMap, sync::Arc};
use validator::{ExtrinsicInMem, ExtrinsicInPool};

/// Extrinsic Validation pool in memory
///
/// Storing extrinsic with in smart pointers for avoiding memory allocation.
///
/// - queue of validated extrinsic
/// - queue of imported extrinsic
/// - queue of extrinsic to be validated
pub struct Pool {
    /// Safrole consensus system state
    pub safrole: Arc<Safrole>,
    /// Extrinsic stored in pool
    pub extrinsic: BTreeMap<OpaqueHash, ExtrinsicInPool>,
    /// Validated extrinsic
    pub memory: BTreeMap<OpaqueHash, Arc<RwLock<ExtrinsicInMem>>>,
    /// extrinsic ready to be packed into block
    pub ready: Vec<OpaqueHash>,
}

impl Pool {
    /// Creates a new extrinsic pool
    pub fn new(safrole: Arc<Safrole>) -> Self {
        Self {
            safrole,
            extrinsic: BTreeMap::new(),
            memory: BTreeMap::new(),
            ready: Vec::new(),
        }
    }

    /// Import extrinsic
    pub fn import(&mut self, block_hash: OpaqueHash, extrinsic: Extrinsic) {
        let Extrinsic {
            assurances,
            disputes,
            preimages,
            guarantees,
            tickets,
        } = extrinsic;

        let ex = ExtrinsicInPool {
            assurances: Arc::new(assurances),
            disputes: Arc::new(disputes),
            preimages: Arc::new(preimages),
            guarantees: Arc::new(guarantees),
            tickets: Arc::new(tickets),
        };

        self.memory.insert(
            block_hash,
            Arc::new(RwLock::new(ExtrinsicInMem {
                assurances: Some(ex.assurances.clone()),
                disputes: Some(ex.disputes.clone()),
                preimages: Some(ex.preimages.clone()),
                guarantees: Some(ex.guarantees.clone()),
                tickets: Some(ex.tickets.clone()),
            })),
        );

        self.extrinsic.insert(block_hash, ex);
    }
}
