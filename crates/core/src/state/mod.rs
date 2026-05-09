//! State of SpaceJam

use crate::{
    AUTH_QUEUE_SIZE, CORES_COUNT, EntropyBuffer, Extrinsic, OpaqueHash, TimeSlot, TrieKey,
    block::History,
    extrinsic::DisputesRecords,
    safrole::{Safrole, Validators},
    service::{AccumulatedQueue, AvailabilityAssignments, Privileges, ReadyQueue, ServiceAccount},
    statistic::Statistics,
};
pub use info::{ServiceField, StateKey, StateKeyInfo, StateKeyLike};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use service::vm::CommitmentMap;
use std::collections::BTreeMap;

pub mod account;
pub mod info;
pub mod key;

/// The state of SpaceJam
///
/// σ = (α, β, γ, δ, η, ι, κ, λ, ρ, τ, φ, χ, ψ, π, θ, ξ)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct State {
    /// The authorization pools (α)
    pub pools: [Vec<OpaqueHash>; CORES_COUNT],

    /// The recent blocks (β)
    pub recent_blocks: History,

    /// State concerning Safrole (γ)
    pub safrole: Safrole,

    /// The prior state of the service accounts (δ)
    pub accounts: BTreeMap<u32, ServiceAccount>,

    /// The entropy accumulator and epochal randomness (η)
    pub entropy: EntropyBuffer,

    /// The validators (ι, κ, λ)
    pub validators: Validators,

    /// The pending reports, per core, which are being made available prior to
    /// accumulation. (ρ)
    pub reports: AvailabilityAssignments,

    /// The current timeslot (τ)
    pub timeslot: TimeSlot,

    /// The authorization queue (φ)
    pub authorization: [[OpaqueHash; AUTH_QUEUE_SIZE]; CORES_COUNT],

    /// The privileged service indices (χ)
    pub privileges: Privileges,

    /// Past judgments (disputes) on work-reports and validators (ψ)
    pub disputes: DisputesRecords,

    /// The activity statistics for the validators (π)
    pub statistics: Statistics,

    /// The accumulation queue (θ)
    pub queue: ReadyQueue,

    /// The accumulation logs (θ)
    pub logs: CommitmentMap,

    /// The accumulation history (ξ)
    pub history: AccumulatedQueue,
}

impl Default for State {
    fn default() -> Self {
        Self {
            pools: Default::default(),
            recent_blocks: Default::default(),
            safrole: Default::default(),
            accounts: Default::default(),
            entropy: Default::default(),
            validators: Default::default(),
            reports: Default::default(),
            timeslot: Default::default(),
            authorization: [[[0; 32]; AUTH_QUEUE_SIZE]; CORES_COUNT],
            privileges: Default::default(),
            disputes: Default::default(),
            statistics: Default::default(),
            queue: Default::default(),
            logs: Default::default(),
            history: Default::default(),
        }
    }
}

impl State {
    /// Get the state pairs
    pub fn pairs(&self, new_epoch: bool, extrinsics: &Extrinsic) -> BTreeMap<TrieKey, Vec<u8>> {
        let mut pairs: BTreeMap<TrieKey, Box<dyn erased_serde::Serialize + Send + Sync>> =
            BTreeMap::new();
        if new_epoch {
            pairs.insert(
                key::PREVIOUS_VALIDATORS,
                Box::new(&self.validators.previous),
            );
            pairs.insert(key::CURRENT_VALIDATORS, Box::new(&self.validators.current));
            pairs.insert(key::SAFROLE, Box::new(&self.safrole));
        }

        if !extrinsics.disputes.is_empty() {
            pairs.insert(key::DISPUTES, Box::new(&self.disputes));
        }

        if !extrinsics.tickets.is_empty() {
            pairs.insert(key::SAFROLE, Box::new(&self.safrole));
        }

        pairs.insert(key::AUTHORIZATION_POOLS, Box::new(&self.pools));
        pairs.insert(key::ENTROPY, Box::new(&self.entropy));
        pairs.insert(key::TIMESLOT, Box::new(&self.timeslot));
        pairs.insert(key::PENDING_REPORTS, Box::new(&self.reports));
        pairs.insert(key::PRIVILEGED_SERVICE, Box::new(&self.privileges));
        pairs.insert(key::ACCUMULATION_LOGS, Box::new(&self.logs));
        pairs.insert(key::ACCUMULATION_QUEUE, Box::new(&self.queue));
        pairs.insert(key::ACCUMULATION_HISTORY, Box::new(&self.history));
        pairs.insert(key::DRAWN_VALIDATORS, Box::new(&self.validators.drawn));
        pairs.insert(key::RECENT_BLOCKS, Box::new(&self.recent_blocks));
        pairs.insert(key::STATISTICS, Box::new(&self.statistics));
        pairs
            .par_iter()
            .map(|(key, value)| (*key, codec::encode(value)))
            .collect()
    }
}
