//! Service account types

use crate::{service::GasLimit, Gas, OpaqueHash, TimeSlot};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;

/// The service accounts (δ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default, Json)]
pub struct ServiceAccount {
    /// The index of the service account (i)
    pub index: u32,

    /// storage of the service account (s)
    pub storage: BTreeMap<Vec<u8>, Vec<u8>>,

    /// The preimage of the service account (p)
    pub preimage: BTreeMap<OpaqueHash, Vec<u8>>,

    /// Preimage lookup dictionary (l)
    pub lookup: BTreeMap<(OpaqueHash, u32), Vec<TimeSlot>>,

    /// The code hash of the service account (c)
    #[json(hex)]
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    pub balance: u64,

    /// The accumulate gas of the service account (g)
    pub accumulate_gas: Gas,

    /// The transfer gas of the service account (m)
    pub transfer_gas: Gas,

    /// The creation time of the service account (t)
    pub creation: u32,

    /// The last update time of the service account (u)
    pub update: u32,

    /// The parent of the service account (p)
    pub parent: u32,

    /// The deposit offset of the service account (f)
    pub offset: u64,

    /// The total number of octets used in storage (o)
    pub total: u64,
}

impl ServiceAccount {
    /// Create a new service account
    pub const fn new(gas: GasLimit) -> Self {
        Self {
            index: 0,
            storage: BTreeMap::new(),
            preimage: BTreeMap::new(),
            lookup: BTreeMap::new(),
            code: [0u8; 32],
            balance: crate::BALANCE_PER_SERVICE,
            accumulate_gas: gas.accumulate,
            transfer_gas: gas.transfer,
            creation: 0,
            update: 0,
            parent: 0,
            offset: 0,
            total: 0,
        }
    }

    /// The threshold of the service account
    pub fn threshold(&self) -> u64 {
        crate::BALANCE_PER_SERVICE
            + crate::BALANCE_PER_ITEM * self.items() as u64
            + crate::BALANCE_PER_OCTET * self.total
    }

    /// Get the present code of the service account
    pub fn code(&self) -> Option<&Vec<u8>> {
        self.preimage.get(&self.code)
    }

    /// The number of items in storage
    pub fn items(&self) -> u32 {
        2 * self.lookup.len() as u32 + self.storage.len() as u32
    }

    /// The state of the service account
    pub fn state(&self) -> ServiceInfo {
        let items = self.items();
        ServiceInfo {
            code: self.code,
            balance: self.balance,
            accumulate: self.accumulate_gas,
            transfer: self.transfer_gas,
            offset: self.offset,
            total: self.total,
            items,
            creation: self.creation,
            update: self.update,
            parent: self.parent,
        }
    }

    /// Get the data of the service account
    pub fn data(&self) -> ServiceData {
        ServiceData {
            accumulate: self.accumulate_gas,
            transfer: self.transfer_gas,
            total: self.total,
            items: self.items(),
            code: self.code,
            balance: self.balance,
        }
    }
}

/// Service info for pvm execution
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Json)]
pub struct ServiceInfo {
    /// The code hash of the service account (c)
    #[json(hex)]
    #[serde(alias = "code_hash")]
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    // #[serde(with = "codec::compact")]
    pub balance: u64,

    /// The minimum required for the on transfer entry-point (m)
    #[serde(alias = "min_item_gas")]
    // #[serde(with = "codec::compact")]
    pub transfer: Gas,

    /// The minimum gas in order to execute the accumulate
    /// entry-point of the service code (g)
    #[serde(alias = "min_memo_gas")]
    // #[serde(with = "codec::compact")]
    pub accumulate: Gas,

    /// The total number of octets used in storage (o)
    #[serde(alias = "bytes")]
    // #[serde(with = "codec::compact")]
    pub total: u64,

    /// The deposit offset of the service account (f)
    #[serde(alias = "deposit_offset")]
    // #[serde(with = "codec::compact")]
    pub offset: u64,

    /// The number of items in storage (i)
    // #[serde(with = "codec::compact")]
    pub items: u32,

    /// The creation time of the service account (t)
    #[serde(alias = "creation_slot")]
    // #[serde(with = "codec::compact")]
    pub creation: u32,

    /// The last update time of the service account (u)
    #[serde(alias = "last_accumulation_slot")]
    // #[serde(with = "codec::compact")]
    pub update: u32,

    /// The parent of the service account (p)
    #[serde(alias = "parent_service")]
    // #[serde(with = "codec::compact")]
    pub parent: u32,
}

/// Service data for state storage codec
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceData {
    /// The code hash of the service account (c)
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    pub balance: u64,

    /// The accumulate gas of the service account (g)
    pub accumulate: u64,

    /// The minimum required for the on transfer entry-point (m)
    pub transfer: Gas,

    /// The total number of octets used in storage (o)
    pub total: u64,

    /// The number of items in storage (i)
    pub items: u32,
}

impl From<ServiceInfo> for ServiceData {
    fn from(state: ServiceInfo) -> Self {
        ServiceData {
            code: state.code,
            balance: state.balance,
            accumulate: state.accumulate,
            transfer: state.transfer,
            total: state.total,
            items: state.items,
        }
    }
}

#[cfg(feature = "crypto")]
mod crypto_impl {
    use super::*;
    use crate::{state::account, TrieKey};
    use std::collections::BTreeSet;

    impl ServiceAccount {
        /// Get all keys of the service account
        pub fn keys(&self, index: u32) -> anyhow::Result<impl Iterator<Item = TrieKey>> {
            let mut keys = BTreeSet::new();
            keys.insert(account::info(index));
            for (key, _) in self.storage.iter() {
                keys.insert(key.to_vec().try_into().map_err(|_| {
                    anyhow::anyhow!(
                        "invalid storage key, expected 31 bytes got {} bytes",
                        key.len()
                    )
                })?);
            }
            for (key, _) in self.preimage.iter() {
                keys.insert(account::preimage(index, *key));
            }
            for ((key, lookup), _) in self.lookup.iter() {
                keys.insert(account::lookup(index, *lookup, *key));
            }
            Ok(keys.into_iter())
        }
    }
}
