//! State context for accumulation

use crate::{
    safrole::ValidatorData,
    service::{Privileges, ServiceAccount},
    OpaqueHash, TimeSlot,
};
use std::collections::BTreeMap;

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

/// External environment specified in spacejam
pub struct Environment {
    /// (η'0) entropy
    pub entropy: OpaqueHash,

    /// (Ht) The current timeslot
    pub timeslot: TimeSlot,
}

impl Environment {
    /// Create a new environment
    pub fn new(entropy: OpaqueHash, timeslot: TimeSlot) -> Self {
        Self { entropy, timeslot }
    }
}
