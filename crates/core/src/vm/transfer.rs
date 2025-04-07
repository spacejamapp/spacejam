//! Deferred transfer related stuffs

use crate::{Gas, ServiceId};
use serde::{Deserialize, Serialize};

/// A deferred transfer item
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

    /*  /// integrate the deferred transfers
    pub fn integrate<V: Vm>(
        accounts: &mut BTreeMap<ServiceId, ServiceAccount>,
        transfers: &[DeferredTransfer],
        slot: TimeSlot,
    ) -> anyhow::Result<Gas> {
        let mut gas_used = 0;
        // Process each account in the intermediate state
        for (service_id, _account) in accounts.clone().into_iter() {
            let transfers = DeferredTransfer::select(transfers, service_id);
            if transfers.is_empty() {
                continue;
            }

            // Invoke PVM's transfer function (Ψ_T) for this service
            // This applies all transfers targeting this service in order
            //
            // TODO: handle the changes of accounts may be using smart pointer.
            let (new_account, gas) = V::transfer(accounts, slot, service_id, &transfers);

            gas_used += gas;
            accounts.insert(service_id, new_account);
        }

        Ok(gas_used)
    } */
}
