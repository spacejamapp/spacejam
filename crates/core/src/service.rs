use crate::{Gas, OpaqueHash, TimeSlot};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;

/// Represents a service info.
///
/// TODO: replace this with the new struct while refactoring tests
#[derive(Debug, Serialize, Deserialize, Json, Clone, PartialEq, Eq)]
pub struct ServiceInfo {
    #[json(hex)]
    pub code_hash: OpaqueHash,
    pub balance: u64,
    pub min_item_gas: Gas,
    pub min_memo_gas: Gas,
    pub bytes: u64,
    pub items: u32,
}

/// The service accounts (δ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct GasLimit {
    /// The minimum gas in order to execute the accumulate
    /// entry-point of the service code (g)
    pub accumulate: Gas,

    /// The minimum required for the on transfer entry-point (m)
    pub transfer: Gas,
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
