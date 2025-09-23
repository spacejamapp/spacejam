//! Virtual machine shared types

use crate::{
    BTreeMap, BandersnatchPublic, BlsPublic, Ed25519Public, EntropyBuffer, Gas, OpaqueHash,
    ServiceId, String, TimeSlot, ValidatorMetadata, Vec,
    service::{Privileges, ServiceAccount, WorkPackage},
    vm::{DeferredTransfer, Operand},
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Data of validators
pub type ValidatorsData = [ValidatorData; 6];

/// The accounts type
pub type Accounts = BTreeMap<ServiceId, ServiceAccount>;

/// Arguments for the authorize invocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizeArgs {
    // (p) the work package
    pub package: WorkPackage,
    // (c) The core index
    pub core_idx: u16,
    // (δ) accounts for historical lookup
    pub accounts: Accounts,
    // (N_t) timeslot for the current operation
    pub timeslot: TimeSlot,
}

/// Arguments for the refine invocation
#[derive(Serialize, Deserialize)]
pub struct RefineArgs {
    // (c) the core index
    pub core: u16,
    // (i) the work item index
    pub index: usize,
    // (p) the work package
    pub package: WorkPackage,
    // (r) the authorizer output
    pub auth_output: Vec<u8>,
    // (ī) all work items' import segments
    pub all_imports: Vec<Vec<Segment>>,
    // (ς) export segment offset
    pub export_offset: u16,
    // (δ) accounts for historical lookup
    pub accounts: Accounts,
    // (N_t) timeslot for the current operation
    pub timeslot: TimeSlot,
}

/// A segment of the import segments
#[derive(Serialize, Deserialize)]
pub struct Segment(#[serde(with = "codec::bytes")] pub [u8; 4104]);

/// Arguments for the accumulate invocation
#[derive(Serialize, Deserialize)]
pub struct AccumulateArgs {
    // (U) The state context
    pub context: AccumulateState,
    // (N_t)  timeslot for the current accumulation
    pub timeslot: TimeSlot,
    // (N_s)  the service id of the caller
    pub service: ServiceId,
    // (N_g)  the gas limit for the current operation
    pub gas: Gas,
    // (O)  the accumulation operands
    pub operands: Vec<Operand>,
}

/// State for the accumulate invocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccumulateState {
    /// d (δ) The accounts
    pub accounts: Accounts,

    /// i (ι) The upcoming validators
    pub validators: ValidatorsData,

    /// p (φ) The authorization queue
    pub authorization: [Vec<OpaqueHash>; crate::CORES_COUNT],

    /// a (χ) The privileged service indices
    pub privileges: Privileges,

    /// (η) The entropy
    pub entropy: EntropyBuffer,
}

/// The accumulated result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Accumulated {
    /// (o) The state context
    pub context: AccumulateState,

    /// (t) The timeslot for the current accumulation
    pub transfers: Vec<DeferredTransfer>,

    /// (b) The output hash of the accumulation
    pub hash: Option<OpaqueHash>,

    /// (u) The gas used
    pub gas: Gas,

    /// (_e) The reason for the accumulation
    pub reason: Reason,
}

/// Represents the ValidatorData structure from ASN.1
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Copy)]
pub struct ValidatorData {
    #[json(hex)]
    pub bandersnatch: BandersnatchPublic,
    #[json(hex)]
    pub ed25519: Ed25519Public,
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub bls: BlsPublic,
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub metadata: ValidatorMetadata,
}

/// The program exit reason.
///
/// As defined per the graypaper (A.2)
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Reason {
    /// The program has halted.
    Halt,

    /// The program has panicked.
    Panic(String),

    /// The invocation completed with a page fault.
    Fault { page: u32 },

    /// The status is unknown.
    HostCall(u32),

    /// The program has run out of gas.
    OOG,

    /// The program is still running.
    #[default]
    Continue,
}
