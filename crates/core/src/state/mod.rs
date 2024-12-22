//! State of SpaceJam

use crate::{
    block::history::BlockInfo,
    extrinsic::{DisputesRecords, TicketsAccumulator, TicketsOrKeys},
    misc::{
        BandersnatchRingCommitment, EntropyBuffer, Gas, OpaqueHash, Statistics, TimeSlot,
        ValidatorData, ValidatorsData,
    },
    work::report::WorkReport,
    CORES_COUNT, EPOCH_LENGTH,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub use storage::Storage;

pub mod key;
mod storage;

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
    pub service_accounts: BTreeMap<u32, ServiceAccount>,

    /// The entropy accumulator and epochal randomness (η)
    pub entropy: EntropyBuffer,

    /// The validators (ι, κ, λ)
    #[serde(flatten)]
    pub validators: Validators,

    /// The pending reports, per core, which are being made available prior to
    /// accumulation. (ρ)
    pub reports: [Option<(WorkReport, TimeSlot)>; CORES_COUNT],

    /// The current timeslot (τ)
    pub timeslot: TimeSlot,

    /// The authorization queue (φ)
    pub authorization: [Vec<OpaqueHash>; CORES_COUNT],

    /// The privileged service indices (χ)
    pub service: ServiceIndex,

    /// Past judgments (disputes) on work-reports and validators (ψ)
    pub disputes: DisputesRecords,

    /// The activity statistics for the validators (π)
    pub statistics: Statistics,

    /// The accumulation queue (θ)
    pub queue: [(Vec<WorkReport>, Vec<OpaqueHash>); EPOCH_LENGTH as usize],

    /// The accumulation history (ξ)
    pub history: [Vec<OpaqueHash>; EPOCH_LENGTH as usize],
}

impl State {
    /// Accumulate the state into a vector of key-value pairs
    pub fn accumulate(&self) -> anyhow::Result<Vec<(OpaqueHash, Vec<u8>)>> {
        let mut kvs = vec![
            (key::AUTHORIZATION_POOLS, codec::encode(&self.pools)?),
            (
                key::AUTHORIZATION_QUEUE,
                codec::encode(&self.authorization)?,
            ),
            (key::RECENT_BLOCKS, codec::encode(&self.recent_blocks)?),
            (key::SAFROLE, codec::encode(&self.safrole)?),
            (key::DISPUTES, codec::encode(&self.disputes)?),
            (key::ENTROPY, codec::encode(&self.entropy)?),
            (key::NEXT_VALIDATORS, codec::encode(&self.validators.next)?),
            (
                key::CURRENT_VALIDATORS,
                codec::encode(&self.validators.current)?,
            ),
            (
                key::PREVIOUS_VALIDATORS,
                codec::encode(&self.validators.previous)?,
            ),
            (key::PENDING_REPORTS, codec::encode(&self.reports)?),
            (key::TIMESLOT, codec::encode(&self.timeslot)?),
            (key::PRIVILEGED_SERVICE, codec::encode(&self.service)?),
            (key::STATISTICS, codec::encode(&self.statistics)?),
            (key::ACCUMULATION_QUEUE, codec::encode(&self.queue)?),
            (key::ACCUMULATION_HISTORY, codec::encode(&self.history)?),
        ];

        for (service, acc) in self.service_accounts.iter() {
            let mut value = Vec::new();
            value.extend_from_slice(&acc.code);
            value.extend_from_slice(&codec::encode(&(
                &acc.balance,
                &acc.gas.accumulate,
                &acc.gas.transfer,
                &acc.total(),
            ))?);
            value.extend_from_slice(&acc.items().to_le_bytes());
            kvs.push((key::account::state(*service), value));

            for (storage, value) in acc.storage.iter() {
                kvs.push((
                    key::account::storage(*service, *storage),
                    codec::encode(value)?,
                ));
            }

            for (preimage, value) in acc.preimage.iter() {
                kvs.push((
                    key::account::preimage(*service, *preimage),
                    codec::encode(value)?,
                ));
            }

            for ((h, lookup), slots) in acc.lookup.iter() {
                kvs.push((
                    key::account::lookup(*service, *lookup, *h),
                    slots.iter().flat_map(|slot| slot.to_le_bytes()).collect(),
                ));
            }
        }

        Ok(kvs)
    }

    /// Calculate the root of the state
    pub fn root(&self, index: usize) -> anyhow::Result<OpaqueHash> {
        let kvs = self.accumulate()?;
        Ok(merkle::trie(&kvs, index))
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

impl Default for Safrole {
    fn default() -> Self {
        Self {
            accumulator: vec![],
            validators: vec![],
            series: TicketsOrKeys::default(),
            ring_commitment: [0u8; 144],
        }
    }
}

/// The service accounts (δ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ServiceAccount {
    /// storage of the service account (s)
    pub storage: BTreeMap<OpaqueHash, Vec<u8>>,

    /// The preimage of the service account (p)
    pub preimage: BTreeMap<OpaqueHash, Vec<u8>>,

    /// Preimage lookup dictionary (l)
    pub lookup: BTreeMap<(OpaqueHash, u32), [TimeSlot; 3]>,

    /// The code hash of the service account (c)
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    pub balance: u64,

    /// The gas limits of the service account (g) and (m)
    #[serde(flatten)]
    pub gas: GasLimit,
}

impl ServiceAccount {
    /// The number of items in storage
    pub fn items(&self) -> u32 {
        2 * self.lookup.len() as u32 + self.storage.len() as u32
    }

    /// total number of octets used in storage
    pub fn total(&self) -> u64 {
        self.lookup
            .iter()
            .map(|((_, z), _)| 81 + *z as u64)
            .chain(self.storage.values().map(|x| 32 + x.len() as u64))
            .sum::<u64>()
    }

    /// The state of the service account
    pub fn state(&self) -> ServiceAccountState {
        ServiceAccountState {
            code: self.code,
            balance: self.balance,
            gas: self.gas.clone(),
            items: self.items(),
            total: self.total(),
        }
    }
}

/// The state of the service account
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ServiceAccountState {
    /// The code hash of the service account (c)
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    pub balance: u64,

    /// The gas limits of the service account (g) and (m)
    #[serde(flatten)]
    pub gas: GasLimit,

    /// The total number of octets used in storage (t)
    pub total: u64,

    /// The number of items in storage (i)
    pub items: u32,
}

/// The gas limits of the service account
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct GasLimit {
    /// The minimum gas in order to execute the accumulate
    /// entry-point of the service code (g)
    pub accumulate: Gas,

    /// The minimum required for the on transfer entry-point (m)
    pub transfer: Gas,
}

/// The validators (ι, κ, λ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct Validators {
    /// The validator keys and metadata to be drawn from next (ι)
    pub next: Vec<ValidatorData>,

    /// The validator keys and metadata currently active (κ)
    pub current: Vec<ValidatorData>,

    /// The validator keys and metadata of the previous epoch (λ)
    pub previous: Vec<ValidatorData>,
}

/// The privileged service indices (χ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct ServiceIndex {
    /// The manager of the service index (m)
    pub manager: u32,

    /// The authorized service indices (a)
    pub authorized: u32,

    /// index of the validator keys and metadata to be drawn
    /// from next (t)
    pub validator: u32,

    /// indices of services which automatically accumulate
    /// in each block together with a basic amount of gas with
    /// which each accumulates.
    pub gas: BTreeMap<u32, Gas>,
}
