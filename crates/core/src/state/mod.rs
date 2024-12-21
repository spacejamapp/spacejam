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

pub mod key;
pub mod storage;

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
        let mut kvs = Vec::new();
        kvs.push((key::AUTHORIZATION_POOLS, codec::encode(&self.pools)?));
        kvs.push((
            key::AUTHORIZATION_QUEUE,
            codec::encode(&self.authorization)?,
        ));
        kvs.push((key::RECENT_BLOCKS, codec::encode(&self.recent_blocks)?));
        kvs.push((key::SAFROLE, codec::encode(&self.safrole)?));
        kvs.push((key::JUDGEMENTS, codec::encode(&self.disputes)?));
        kvs.push((key::ENTROPY, codec::encode(&self.entropy)?));
        kvs.push((key::NEXT_VALIDATORS, codec::encode(&self.validators.next)?));
        kvs.push((
            key::CURRENT_VALIDATORS,
            codec::encode(&self.validators.current)?,
        ));
        kvs.push((
            key::PREVIOUS_VALIDATORS,
            codec::encode(&self.validators.previous)?,
        ));
        kvs.push((key::PENDING_REPORTS, codec::encode(&self.reports)?));
        kvs.push((key::TIMESLOT, codec::encode(&self.timeslot)?));
        kvs.push((key::PRIVILEGED_SERVICE, codec::encode(&self.service)?));
        kvs.push((key::STATISTICS, codec::encode(&self.statistics)?));
        kvs.push((key::ACCUMULATION_QUEUE, codec::encode(&self.queue)?));
        kvs.push((key::ACCUMULATION_HISTORY, codec::encode(&self.history)?));

        // Encode the service accounts
        //
        // TODO: confirm what is a_i and is we need to encode a_l to the state
        for (service, acc) in self.service_accounts.iter() {
            let mut value = Vec::new();
            value.extend_from_slice(&acc.code);
            value.extend_from_slice(
                &codec::encode(&(
                    &acc.balance,
                    &acc.gas.accumulate,
                    &acc.gas.transfer,
                    &acc.lookup,
                ))?[..8],
            );
            value.extend_from_slice(&service.to_le_bytes());
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
                    slots
                        .iter()
                        .map(|slot| slot.to_le_bytes())
                        .flatten()
                        .collect(),
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
    pub balance: u32,

    /// The gas limits of the service account (g) and (m)
    #[serde(flatten)]
    pub gas: GasLimit,
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Validators {
    /// The validator keys and metadata to be drawn from next (ι)
    pub next: Vec<ValidatorData>,

    /// The validator keys and metadata currently active (κ)
    pub current: Vec<ValidatorData>,

    /// The validator keys and metadata of the previous epoch (λ)
    pub previous: Vec<ValidatorData>,
}

/// The privileged service indices (χ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
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
