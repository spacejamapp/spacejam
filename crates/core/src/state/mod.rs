//! State of SpaceJam

use crate::{
    block::{History, HistoryJson},
    extrinsic::{DisputesRecords, DisputesRecordsJson},
    safrole::{Safrole, SafroleJson, Validators, ValidatorsJson},
    service::{
        AccumulatedQueue, AvailabilityAssignmentJson, AvailabilityAssignments, Privileges,
        PrivilegesJson, ReadyQueue, ReadyReportJson, ServiceAccount,
    },
    statistic::{Statistics, StatisticsJson},
    EntropyBuffer, OpaqueHash, TimeSlot, CORES_COUNT,
};
pub use info::{ServiceField, StateKey, StateKeyInfo, StateKeyLike};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;

pub mod account;
pub mod info;
pub mod key;

/// The state of SpaceJam
///
/// σ = (α, β, γ, δ, η, ι, κ, λ, ρ, τ, φ, χ, ψ, π, θ, ξ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default, Json)]
pub struct State {
    /// The authorization pools (α)
    #[json(Vec<Vec<String>>)]
    pub pools: [Vec<OpaqueHash>; CORES_COUNT],

    /// The recent blocks (β)
    #[json(nested)]
    pub recent_blocks: History,

    /// State concerning Safrole (γ)
    #[serde(flatten)]
    #[json(nested)]
    pub safrole: Safrole,

    /// The prior state of the service accounts (δ)
    pub accounts: BTreeMap<u32, ServiceAccount>,

    /// The entropy accumulator and epochal randomness (η)
    #[json(Vec<String>)]
    pub entropy: EntropyBuffer,

    /// The validators (ι, κ, λ)
    #[serde(flatten)]
    #[json(nested)]
    pub validators: Validators,

    /// The pending reports, per core, which are being made available prior to
    /// accumulation. (ρ)
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    pub reports: AvailabilityAssignments,

    /// The current timeslot (τ)
    pub timeslot: TimeSlot,

    /// The authorization queue (φ)
    #[json(Vec<Vec<String>>)]
    pub authorization: [Vec<OpaqueHash>; CORES_COUNT],

    /// The privileged service indices (χ)
    #[json(nested)]
    pub privileges: Privileges,

    /// Past judgments (disputes) on work-reports and validators (ψ)
    #[json(nested)]
    pub disputes: DisputesRecords,

    /// The activity statistics for the validators (π)
    #[json(nested)]
    pub statistics: Statistics,

    /// The accumulation queue (θ)
    #[json(Vec<Vec<ReadyReportJson>>)]
    pub queue: ReadyQueue,

    /// The accumulation history (ξ)
    #[json(Vec<Vec<String>>)]
    pub history: AccumulatedQueue,
}
