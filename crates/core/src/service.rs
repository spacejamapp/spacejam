use crate::{Gas, OpaqueHash, ServiceId, TimeSlot};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;

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
            total: self.total(),
            items: self.items(),
        }
    }
}

/// The state of the service account
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Json)]
pub struct ServiceAccountState {
    /// The code hash of the service account (c)
    #[json(hex)]
    #[serde(alias = "code_hash")]
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    pub balance: u64,

    /// The gas limits of the service account (g) and (m)
    #[serde(flatten)]
    #[json(nested)]
    pub gas: GasLimit,

    /// The total number of octets used in storage (o)
    #[serde(alias = "bytes")]
    pub total: u64,

    /// The number of items in storage (i)
    pub items: u32,
}

/// The gas limits of the service account
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default, Json)]
pub struct GasLimit {
    /// The minimum gas in order to execute the accumulate
    /// entry-point of the service code (g)
    #[serde(alias = "min_memo_gas")]
    pub accumulate: Gas,

    /// The minimum required for the on transfer entry-point (m)
    #[serde(alias = "min_item_gas")]
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

/// Represents a service item.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ServiceItem {
    /// The id of the service item
    pub id: ServiceId,

    /// The info of the service item
    #[json(nested)]
    pub info: ServiceAccountState,
}
