//! The virtual machine interfaces of SpaceJam

use crate::{
    service::{ServiceAccount, WorkExecResult},
    Gas, OpaqueHash, ServiceId, TimeSlot,
};
pub use context::StateContext;
use std::collections::BTreeMap;

mod context;

/// Service to hash commitment map
pub type CommitmentMap = BTreeMap<ServiceId, OpaqueHash>;

/// The virtual machine interface
pub trait Vm {
    /// (ΨA): single step state transition invocation
    fn accumulate(
        // (U) The state context
        _context: StateContext,
        // (N_t)  timeslot for the current accumulation
        _slot: TimeSlot,
        // (N_s)  the service id of the caller
        _service_id: ServiceId,
        // (N_g)  the gas limit for the current operation
        _gas_limit: Gas,
        // (O)  the accumulation operands
        _operands: Vec<Operand>,
    ) -> AccumulateResult {
        Default::default()
    }

    /// (ΨT): on-transfer invocation
    fn transfer(
        // (δ) The account storage
        _accounts: &BTreeMap<ServiceId, ServiceAccount>,
        // (N_t)  timeslot for the current accumulation
        _slot: TimeSlot,
        // (N_s)  the service id of the caller
        _service_id: ServiceId,
        // (T)  the deferred transfers
        _transfers: &[DeferredTransfer],
    ) -> (ServiceAccount, Gas) {
        (ServiceAccount::default(), 0)
    }
}

/// A deferred transfer item
#[derive(Debug, PartialEq, Eq, Clone, Default)]
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

    /// integrate the deferred transfers
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
    }
}

/// An operand of the accumulation
///
/// defined per GP (12.19)
pub struct Operand {
    /// (d) The work execution result
    pub data: WorkExecResult,

    /// (e) The erasure root
    pub erasure_root: OpaqueHash,

    /// (o) The authorizer output
    pub authorizer_output: Vec<u8>,

    /// (y) The payload blob hash
    pub payload: OpaqueHash,

    /// (h) The hash of the work package
    pub hash: OpaqueHash,

    /// (n) The accumulate gas
    pub gas: Gas,
}

/// The accumulate result of (ΨA)
#[derive(Default)]
pub struct AccumulateResult {
    /// (o) The state context
    pub context: StateContext,

    /// (t) The timeslot for the current accumulation
    pub transfers: Vec<DeferredTransfer>,

    /// (b) The output hash of the accumulation
    pub hash: Option<OpaqueHash>,

    /// (u) The gas used
    pub gas: Gas,
}

impl Vm for () {}
