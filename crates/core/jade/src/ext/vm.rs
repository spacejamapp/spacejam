//! Virtual machine shared types

use crate::{
    BandersnatchPublic, BlsPublic, Ed25519Public, EntropyBuffer, OpaqueHash, ServiceId, TimeSlot,
    ValidatorMetadata,
    service::{Privileges, ServiceAccount, WorkPackage},
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;

/// Data of validators
pub type ValidatorsData = [ValidatorData; 6];

/// The accounts type
pub type Accounts = BTreeMap<ServiceId, ServiceAccount>;

/// Arguments for the authorize invocation
#[repr(C)]
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
#[repr(C)]
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
    pub all_imports: Vec<Vec<[u8; 4104]>>,
    // (ς) export segment offset
    pub export_offset: u16,
    // (δ) accounts for historical lookup
    pub accounts: Accounts,
    // (N_t) timeslot for the current operation
    pub timeslot: TimeSlot,
}

/// Arguments for the accumulate invocation
#[repr(C)]
pub struct AccumulateArgs {}

/// State for the accumulate invocation
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
