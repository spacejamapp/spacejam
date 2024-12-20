//! State of SpaceJam

use crate::{
    block::history::BlockInfo,
    extrinsic::{TicketsAccumulator, TicketsOrKeys},
    misc::{
        BandersnatchRingCommitment, EntropyBuffer, OpaqueHash, Statistics, TimeSlot, ValidatorData,
        ValidatorsData,
    },
    CORES_COUNT,
};
use serde::{Deserialize, Serialize};

/// The state of SpaceJam
///
/// σ = (α, β, γ, δ, η, ι, κ, λ, ρ, τ, φ, χ, ψ, π, θ, ξ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct State {
    /// The authorization pools (α)
    pub pools: [Vec<OpaqueHash>; CORES_COUNT],

    /// The recent blocks (β)
    pub recent_blocks: Vec<BlockInfo>,

    /// State concerning Safrole (γ)
    #[serde(flatten)]
    pub safrole: Safrole,

    /// The prior state of the service accounts (δ)
    ///
    /// TODO: to be confirmed, see also the report implementation.
    pub service_accounts: ServiceAccounts,

    /// The entropy accumulator and epochal randomness (η)
    pub entropy: EntropyBuffer,

    /// The validators (ι, κ, λ)
    #[serde(flatten)]
    pub validators: Validators,

    /// The pending reports, per core, which are being made available prior to
    /// accumulation. (ρ)
    pub reports: [PendingReports; CORES_COUNT],

    /// The current timeslot (τ)
    pub timeslot: TimeSlot,

    /// The authorization queue (φ)
    pub authorization: Vec<()>,

    /// The privileged service indices (χ)
    pub service: Vec<()>,

    /// Past judgments on work-reports and validators (ψ)
    pub judgments: Vec<()>,

    /// The activity statistics for the validators (π)
    pub statistics: Statistics,

    /// The accumulation queue (θ)
    pub queue: Vec<()>,

    /// The accumulation history (ξ)
    pub history: Vec<()>,
}

impl State {
    /// The root of the state
    ///
    /// H_r = M_σ(σ)
    pub fn root(&self) -> OpaqueHash {
        todo!()
    }
}

/// Safrole consensus state
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Safrole {
    /// Sealing-key contest ticket accumulator (gamma_a)
    pub accumulator: TicketsAccumulator,
    /// Next epoch's validators (gamma_k)
    pub validators: ValidatorsData,
    /// Sealing-key series of the current epoch (gamma_s)
    pub series: TicketsOrKeys,
    /// Bandersnatch ring commitment (gamma_z)
    #[serde(with = "codec::bytes")]
    pub ring_commitment: BandersnatchRingCommitment,
}

/// The service accounts (δ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ServiceAccounts {
    /// The post-preimage integration, pre-accumulation intermediate state.
    pub preimage: (),

    /// The post-accumulation, pre-transfer intermediate state.
    pub accumulation: (),
}

/// The validators (ι, κ, λ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Validators {
    /// The validator keys and metadata to be drawn from next (ι)
    pub next: Vec<ValidatorData>,

    /// The validator keys and metadata currently active (κ)
    pub current: Vec<ValidatorData>,

    /// The validator keys and metadata of the previous epoch (λ)
    pub previous: Vec<ValidatorData>,
}

/// The pending reports, being made available prior to accumulation.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PendingReports {
    /// The post-judgment, pre-guarantees-extrinsic intermediate state.
    pub judgement: (),
    /// The post-guarantees-extrinsic, pre-accumulation intermediate state.
    pub guarantees: (),
}
