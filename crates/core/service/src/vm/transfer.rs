//! Primitives for the transfer invocation

use crate::{Gas, ServiceId, Vec};
use serde::{Deserialize, Serialize};

/// (12.14) A deferred transfer item
#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct DeferredTransfer {
    /// (s) The sender
    pub sender: ServiceId,

    /// (d) The destination
    pub recipient: ServiceId,

    /// (a) The amount
    pub amount: u64,

    /// (m) The memo
    pub memo: Vec<u8>,

    /// (g) The gas limit
    pub gas_limit: Gas,
}

impl DeferredTransfer {
    /// (R): Select transfers for a given destination service
    pub fn select(transfers: &[DeferredTransfer], dest: ServiceId) -> Vec<DeferredTransfer> {
        let mut transfers = transfers.to_vec();
        transfers.sort_by_key(|t| t.sender);
        transfers
            .iter()
            .filter(|t| t.recipient == dest)
            .cloned()
            .collect()
    }
}
