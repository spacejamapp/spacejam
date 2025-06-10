//! Service account types

use crate::{service::GasLimit, Gas, OpaqueHash, TimeSlot};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;

/// The service accounts (δ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct ServiceAccount {
    /// storage of the service account (s)
    pub storage: BTreeMap<Vec<u8>, Vec<u8>>,

    /// The preimage of the service account (p)
    pub preimage: BTreeMap<OpaqueHash, Vec<u8>>,

    /// Preimage lookup dictionary (l)
    pub lookup: BTreeMap<(OpaqueHash, u32), Vec<TimeSlot>>,

    /// The code hash of the service account (c)
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    pub balance: u64,

    /// The gas limits of the service account (g) and (m)
    #[serde(flatten)]
    pub gas: GasLimit,
}

impl ServiceAccount {
    /// Create a new service account
    pub const fn new(gas: GasLimit) -> Self {
        Self {
            storage: BTreeMap::new(),
            preimage: BTreeMap::new(),
            lookup: BTreeMap::new(),
            code: [0u8; 32],
            balance: crate::BALANCE_PER_SERVICE,
            gas,
        }
    }

    /// The threshold of the service account
    pub fn threshold(&self) -> u64 {
        crate::BALANCE_PER_SERVICE
            + crate::BALANCE_PER_ITEM * self.items() as u64
            + crate::BALANCE_PER_OCTET * self.total()
    }

    /// Get the present code of the service account
    pub fn code(&self) -> Option<&Vec<u8>> {
        self.preimage.get(&self.code)
    }

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
        let items = self.items();
        let total = self.total();
        ServiceAccountState {
            code: self.code,
            balance: self.balance,
            threshold: self.threshold(),
            accumulate: self.gas.accumulate,
            transfer: self.gas.transfer,
            total,
            items,
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
    #[serde(with = "codec::compact")]
    pub balance: u64,

    /// The threshold of the service account (t)
    #[serde(with = "codec::compact", default)]
    pub threshold: u64,

    /// The minimum gas in order to execute the accumulate
    /// entry-point of the service code (g)
    #[serde(alias = "min_memo_gas")]
    #[serde(with = "codec::compact")]
    pub accumulate: Gas,

    /// The minimum required for the on transfer entry-point (m)
    #[serde(alias = "min_item_gas")]
    #[serde(with = "codec::compact")]
    pub transfer: Gas,

    /// The total number of octets used in storage (o)
    #[serde(alias = "bytes")]
    #[serde(with = "codec::compact")]
    pub total: u64,

    /// The number of items in storage (i)
    #[serde(with = "codec::compact")]
    pub items: u32,
}

impl ServiceAccountState {
    /// The minimum balance which the service must satisfy. (t)
    pub const fn threshold(&self) -> u64 {
        crate::BALANCE_PER_SERVICE
            + crate::BALANCE_PER_ITEM * self.items as u64
            + crate::BALANCE_PER_OCTET * self.total
    }
}
