//! Accumulation related types

use account::Accounts;
use pvm::{Account, AccumulateState};
use score::{
    Gas, OpaqueHash, ServiceId,
    safrole::ValidatorsData,
    service::{AccumulatedQueue, Privileges, ReadyQueue, WorkReport},
    statistic::{AccumulationRecord, ServiceActivityRecord},
    vm::{CommitmentMap, DeferredTransfer},
};
use std::collections::{BTreeMap, HashSet};

/// The result of accumulation with PVM
///
/// - N: the number of work-results accumulated.
/// - U: A posterior state-context.
/// - \[T\]: resultant deferred-transfers
/// - B: accumulation-output pairings.
/// - U: the total gas used
#[derive(Clone)]
pub struct Accumulated<R: Accounts> {
    /// (i) the number of work-results accumulated.
    pub accumulated: usize,

    /// (o) A posterior state-context.
    pub context: AccumulateState<R>,

    /// (t) The resultant deferred-transfers
    pub transfers: Vec<DeferredTransfer>,

    /// (b) The accumulation-output pairings.
    pub pairings: CommitmentMap,

    /// (u) The total gas used
    pub gas: BTreeMap<ServiceId, Gas>,
}

impl<R: Accounts> Accumulated<R> {
    /// Create a new accumulated.
    pub fn new(context: AccumulateState<R>) -> Self {
        Self {
            accumulated: 0,
            context,
            transfers: vec![],
            pairings: BTreeMap::new(),
            gas: BTreeMap::new(),
        }
    }

    /// Get the service records
    pub fn records(
        &mut self,
        accumulatable: &[WorkReport],
    ) -> BTreeMap<ServiceId, ServiceActivityRecord> {
        let mut records: BTreeMap<ServiceId, ServiceActivityRecord> = BTreeMap::new();
        for report in accumulatable {
            for result in &report.results {
                let record = records.entry(result.service_id).or_default();
                record.accumulate_count += 1;
                if record.accumulate_gas_used == 0 {
                    record.accumulate_gas_used = *self.gas.get(&result.service_id).unwrap_or(&0);
                }
            }
        }

        for transfer in self.transfers.iter() {
            if records.contains_key(&transfer.recipient)
                || !self.gas.contains_key(&transfer.recipient)
            {
                continue;
            }

            let record = records.entry(transfer.recipient).or_default();
            if record.accumulate_gas_used == 0 {
                record.accumulate_gas_used = *self.gas.get(&transfer.recipient).unwrap_or(&0);
            }
        }

        // update the last update time of the accounts
        for service in records.keys() {
            if let Some(account) = self.context.accounts.get(*service) {
                account.set_update(self.context.timeslot);
            }
        }

        records
    }

    /// Get the accumulation root
    ///
    /// see also (7.7) in the graypaper
    pub fn root(&self) -> OpaqueHash {
        let mut sorted_pairs: Vec<_> = self.pairings.iter().collect();
        sorted_pairs.sort_by_key(|(service_id, _)| *service_id);

        let leaves = sorted_pairs
            .into_iter()
            .map(|(service, commit)| {
                let mut leaf = Vec::new();
                leaf.extend_from_slice(&service.to_le_bytes());
                leaf.extend_from_slice(commit);
                leaf
            })
            .collect::<Vec<_>>();

        crypto::merkle::kroot(leaves)
    }

    /// Apply the deferred transfers to the accounts
    pub fn defer_transfers(&mut self) {
        for transfer in self.transfers.iter() {
            if let Some(dest) = self.context.accounts.get(transfer.recipient) {
                *dest.balance_mut() += transfer.amount;
            }
        }
    }
}

impl<R: Accounts> From<&Accumulated<R>> for AccumulationRecord {
    fn from(accumulated: &Accumulated<R>) -> Self {
        // FIXME: track the affected services
        let affected_services: HashSet<_> = accumulated
            .context
            .accounts
            .accounts()
            .keys()
            .cloned()
            .collect();

        AccumulationRecord {
            work_reports_processed: accumulated.accumulated,
            total_gas_used: accumulated.gas.values().sum(),
            services_affected: affected_services.len(),
            commitment_count: accumulated.pairings.len(),
        }
    }
}

/// The accumulation result used in the runtime
pub struct Accumulation<R: Accounts> {
    /// (r) The accumulate root
    pub root: OpaqueHash,

    /// (θ') The ready queue
    pub ready_queue: ReadyQueue,

    /// (ξ') The accumulated queue
    pub accumulated_queue: AccumulatedQueue,

    /// (δ‡) The accounts
    pub accounts: R,

    /// (χ') The privileges
    pub privileges: Privileges,

    /// (ι') The validators to be drawn
    pub validators: ValidatorsData,

    /// (πS') The service records
    pub records: BTreeMap<ServiceId, ServiceActivityRecord>,

    /// (θ) The accumulation logs
    pub logs: CommitmentMap,
}
