//! State of SpaceJam

use crate::{
    block::BlockInfo,
    extrinsic::DisputesRecords,
    safrole::{Safrole, Validators},
    service::{AccumulatedQueue, AvailabilityAssignments, Privileges, ReadyQueue, ServiceAccount},
    statistic::Statistics,
    EntropyBuffer, OpaqueHash, TimeSlot, CORES_COUNT,
};
pub use info::{ServiceField, StateKey, StateKeyInfo, StateKeyLike};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod account;
pub mod info;
pub mod key;

/// The state of SpaceJam
///
/// σ = (α, β, γ, δ, η, ι, κ, λ, ρ, τ, φ, χ, ψ, π, θ, ξ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct State {
    /// The authorization pools (α)
    pub pools: [Vec<OpaqueHash>; CORES_COUNT],

    /// The recent blocks (β)
    pub recent_blocks: Vec<BlockInfo>,

    /// State concerning Safrole (γ)
    #[serde(flatten)]
    pub safrole: Safrole,

    /// The prior state of the service accounts (δ)
    pub accounts: BTreeMap<u32, ServiceAccount>,

    /// The entropy accumulator and epochal randomness (η)
    pub entropy: EntropyBuffer,

    /// The validators (ι, κ, λ)
    #[serde(flatten)]
    pub validators: Validators,

    /// The pending reports, per core, which are being made available prior to
    /// accumulation. (ρ)
    pub reports: AvailabilityAssignments,

    /// The current timeslot (τ)
    pub timeslot: TimeSlot,

    /// The authorization queue (φ)
    pub authorization: [Vec<OpaqueHash>; CORES_COUNT],

    /// The privileged service indices (χ)
    pub privileges: Privileges,

    /// Past judgments (disputes) on work-reports and validators (ψ)
    pub disputes: DisputesRecords,

    /// The activity statistics for the validators (π)
    pub statistics: Statistics,

    /// The accumulation queue (θ)
    pub queue: ReadyQueue,

    /// The accumulation history (ξ)
    pub history: AccumulatedQueue,
}
