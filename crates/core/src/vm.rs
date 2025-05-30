//! Virtual machine interfaces

use crate::{
    safrole::ValidatorData,
    service::{AccumulatedQueue, Privileges, ReadyQueue, ServiceAccount},
    OpaqueHash,
};
use crate::{service::WorkExecResult, Gas, ServiceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The commitment map
pub type CommitmentMap = BTreeMap<ServiceId, OpaqueHash>;

/// The state context for accumulation
#[derive(Default, Clone)]
pub struct StateContext {
    /// d (δ) The accounts
    pub accounts: BTreeMap<u32, ServiceAccount>,

    /// i (ι) The upcoming validators
    pub validators: Vec<ValidatorData>,

    /// q (φ) The authorization queue
    pub authorization: [Vec<OpaqueHash>; crate::CORES_COUNT],

    /// χ (χ) The privileged service indices
    pub privileges: Privileges,
}

/// The result of the PVM execution
///
/// - N: the number of work-results accumulated.
/// - U: A posterior state-context.
/// - \[T\]: resultant deferred-transfers
/// - B: accumulation-output pairings.
/// - U: the total gas used
#[derive(Default, Clone)]
pub struct Accumulated {
    /// (i) the number of work-results accumulated.
    pub accumulated: usize,

    /// (o) A posterior state-context.
    pub context: StateContext,

    /// (t) The resultant deferred-transfers
    pub transfers: Vec<DeferredTransfer>,

    /// (b) The accumulation-output pairings.
    pub pairings: CommitmentMap,

    /// (u) The total gas used
    pub gas: BTreeMap<ServiceId, Gas>,
}

/// The accumulation result used in the runtime
pub struct Accumulation {
    /// (r) The accumulate root
    pub root: OpaqueHash,

    /// (θ') The ready queue
    pub ready_queue: ReadyQueue,

    /// (ξ') The accumulated queue
    pub accumulated_queue: AccumulatedQueue,

    /// (δ') The accounts
    pub accounts: BTreeMap<u32, ServiceAccount>,

    /// (χ) The privileges
    pub privileges: Privileges,
}

/// An operand of the accumulation
///
/// NOTE: we are currently following the order of jam-types instead
/// of graypaper.
///
/// defined per GP (12.19)
#[derive(Serialize, Deserialize, Debug)]
pub struct Operand {
    /// (h) The hash of the work package
    pub hash: OpaqueHash,

    /// (e) The erasure root
    pub erasure_root: OpaqueHash,

    /// (a) anchor header hash
    pub anchor: OpaqueHash,

    /// (o) The authorizer output
    pub authorizer_output: Vec<u8>,

    /// (y) The payload blob hash
    pub payload: OpaqueHash,

    /*
    // JAM_TYPES currently does not include this field
    /// (g) The accumulate gas
    pub gas: Gas,
    */
    /// (d) The work execution result
    pub data: WorkExecResult,
}

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
}

#[cfg(test)]
mod accumulate_item {

    use jam_types::{
        AccumulateItem, AuthOutput, AuthorizerHash, Encode, PayloadHash, SegmentTreeRoot,
        WorkOutput, WorkPackageHash,
    };

    use super::*;

    #[test]
    fn operand_codec() {
        let operand = Operand {
            hash: [0; 32],
            erasure_root: [0; 32],
            anchor: [0; 32],
            authorizer_output: vec![0; 32],
            payload: [0; 32],
            data: WorkExecResult::Ok(vec![0; 32]),
        };

        let accumulate_item = AccumulateItem {
            authorizer_hash: AuthorizerHash([0; 32]),
            package: WorkPackageHash([0; 32]),
            exports_root: SegmentTreeRoot([0; 32]),
            auth_output: AuthOutput(vec![0; 32]),
            payload: PayloadHash([0; 32]),
            result: Ok(WorkOutput(vec![0; 32])),
        };

        let space_encoded = codec::encode(&operand).unwrap();
        let jam_encoded = accumulate_item.encode();
        assert_eq!(space_encoded, jam_encoded);
    }
}
