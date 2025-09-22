//! Primitives for the accumulate invocation

use crate::{
    EntropyBuffer, OpaqueHash,
    account::Accounts,
    safrole::ValidatorsData,
    service::{AccumulatedQueue, Privileges, ReadyQueue},
    statistic::ServiceActivityRecord,
    vm::DeferredTransfer,
};
use crate::{Gas, ServiceId, service::WorkExecResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The commitment map
pub type CommitmentMap = BTreeMap<ServiceId, OpaqueHash>;

/// The salt for the index function
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexSalt {
    /// (N_s)  the service id of the caller
    #[serde(with = "codec::compact")]
    pub service: u32,

    /// (η) The entropy
    pub entropy: [u8; 32],

    /// (N_t)  timeslot for the current accumulation
    #[serde(with = "codec::compact")]
    pub timeslot: u32,
}

/// The state context used in pvm accumulation
#[derive(Clone)]
pub struct AccumulateState<R: Accounts> {
    /// d (δ) The accounts
    pub accounts: R,

    /// i (ι) The upcoming validators
    pub validators: ValidatorsData,

    /// p (φ) The authorization queue
    pub authorization: [Vec<OpaqueHash>; crate::CORES_COUNT],

    /// a (χ) The privileged service indices
    pub privileges: Privileges,

    /// (η) The entropy
    pub entropy: EntropyBuffer,
}

impl<R: Accounts> AccumulateState<R> {
    /// Get the code hash of an account
    pub fn code_hash(&mut self, service: ServiceId) -> Option<OpaqueHash> {
        self.accounts.code_hash(service)
    }

    /// Share preimages for the services in the state context
    pub fn code(&mut self, service: ServiceId) -> Option<Vec<u8>> {
        self.accounts.blob(service)
        // self.accounts.get(service)?.account().code().cloned()
        // TODO: The logic below is correct, but we need to match
        // the test vectors atm.
        /* let hash = self.accounts.get(service)?.code();
        for account in self.accounts.accounts().values() {
            if account.code() != hash {
                continue;
            }

            if let Some(code) = account.account().code() {
                return Some(code.clone());
            }
        }

        None */
    }
}

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
    pub fn records(&self) -> BTreeMap<ServiceId, ServiceActivityRecord> {
        let mut records = BTreeMap::new();

        // Include all services that exist in accounts
        for service_id in self.context.accounts.services() {
            records.insert(service_id, ServiceActivityRecord::default());
        }

        // Update gas usage for services that actually executed
        for (service, gas) in self.gas.iter() {
            tracing::debug!("Service {} used {} gas during accumulation", service, gas);
            if let Some(record) = records.get_mut(service) {
                record.accumulate_gas_used = *gas;
            } else {
                // Service executed but doesn't exist in accounts (shouldn't happen)
                records.insert(
                    *service,
                    ServiceActivityRecord {
                        accumulate_gas_used: *gas,
                        ..Default::default()
                    },
                );
            }
        }

        records
    }

    /// Get the accumulation root
    ///
    /// see also (7.7) in the graypaper
    #[cfg(feature = "merkle")]
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

    /// (Xt) The transfer statistics: (service_id -> (transfer_count, gas_used))
    pub transfers: BTreeMap<ServiceId, (usize, Gas)>,

    /// (θ) The accumulation logs
    pub logs: CommitmentMap,
}

/// The accumulate params for the accumulation
#[derive(Serialize, Deserialize, Debug)]
pub struct AccumulateParams {
    /// (N_t)  timeslot for the current accumulation
    #[serde(with = "codec::compact")]
    pub slot: u32,

    /// (N_s)  the service id of the caller
    #[serde(with = "codec::compact")]
    pub id: u32,

    /// (|o|) The count of operands
    #[serde(with = "codec::compact")]
    pub results: u32,
}

/// An operand of the accumulation
///
/// NOTE: we are currently following the order of jam-types instead
/// of graypaper.
///
/// defined per GP (12.19)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operand {
    /// (p) The hash of the work package
    pub package: OpaqueHash,

    /// (e) The root of the segment tree which was generated by the work-package
    /// in which the work-item which gave this result was placed.
    pub exports_root: OpaqueHash,

    /// (a) The hash of the authorizer which authorized the execution of the
    /// work-package which generated this result.
    pub authorizer_hash: OpaqueHash,

    /// (y) The payload blob hash
    pub payload: OpaqueHash,

    // JAM_TYPES currently does not include this field
    /// (g) The accumulate gas
    #[serde(with = "codec::compact")]
    pub gas: Gas,

    /// (d) The work execution result
    pub data: WorkExecResult,

    /// (o) The output of the Is-Authorized logic which authorized the execution
    /// of the work-package which generated this result.
    pub auth_output: Vec<u8>,
}
