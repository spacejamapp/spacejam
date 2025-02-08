//! State of SpaceJam

use crate::{
    block::BlockInfo,
    extrinsic::DisputesRecords,
    safrole::Safrole,
    safrole::Validators,
    service::{AvailabilityAssignments, WorkReport},
    service::{ServiceAccount, ServiceIndex},
    statistic::Statistics,
    EntropyBuffer, OpaqueHash, TimeSlot, CORES_COUNT, EPOCH_LENGTH,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub use storage::Storage;

pub mod account;
pub mod key;
pub mod storage;

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

/// Get the diff of the accounts
pub fn accounts(
    accounts: &BTreeMap<u32, ServiceAccount>,
) -> anyhow::Result<Vec<(OpaqueHash, Vec<u8>)>> {
    let mut diff = vec![];
    for (index, account) in accounts {
        // set info
        let mut value = Vec::new();
        let info = account.state();
        value.extend_from_slice(&info.code);
        value.extend_from_slice(&codec::encode(&(
            &info.balance,
            &info.gas.accumulate,
            &info.gas.transfer,
            &info.total,
        ))?);
        value.extend_from_slice(&info.items.to_le_bytes());
        diff.push((account::info(*index), value));

        // set storage
        for (key, value) in &account.storage {
            diff.push((account::storage(*index, *key), value.clone()));
        }

        // set preimage
        for (key, value) in &account.preimage {
            diff.push((account::preimage(*index, *key), value.clone()));
        }

        // set lookup
        for ((key, lookup), slots) in &account.lookup {
            diff.push((
                account::lookup(*index, *lookup, *key),
                slots
                    .iter()
                    .flat_map(|slot| slot.to_le_bytes())
                    .collect::<Vec<u8>>(),
            ));
        }
    }

    Ok(diff)
}
