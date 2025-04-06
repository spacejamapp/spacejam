//! State context for accumulation

use crate::{
    safrole::ValidatorData,
    service::{Privileges, ServiceAccount},
    OpaqueHash,
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
