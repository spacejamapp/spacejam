//! Service account types

use crate::{BTreeMap, Gas, OpaqueHash, TimeSlot, Vec, service::GasLimit};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use {crate::String, spacejson::Json};

/// The service accounts (δ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct ServiceAccount {
    /// The index of the service account (i)
    pub index: u32,

    /// storage of the service account (s)
    pub storage: BTreeMap<Vec<u8>, Vec<u8>>,

    /// The preimage of the service account (p)
    pub preimage: BTreeMap<OpaqueHash, Vec<u8>>,

    /// Preimage lookup dictionary (l)
    pub lookup: BTreeMap<(OpaqueHash, u32), Vec<TimeSlot>>,

    /// The info of the service account
    #[cfg_attr(feature = "json", json(nested))]
    pub info: ServiceInfo,
}

impl ServiceAccount {
    /// Create a new service account
    pub fn new(gas: GasLimit) -> Self {
        Self {
            index: 0,
            storage: BTreeMap::new(),
            preimage: BTreeMap::new(),
            lookup: BTreeMap::new(),
            info: ServiceInfo {
                transfer: gas.transfer,
                accumulate: gas.accumulate,
                ..Default::default()
            },
        }
    }

    /// The state of the service account
    pub fn state(&self) -> ServiceInfo {
        self.info.clone()
    }
}

/// Service info for pvm execution
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct ServiceInfo {
    /// The version of the service account (v)
    pub version: u8,

    /// The code hash of the service account (c)
    #[cfg_attr(feature = "json", json(hex))]
    #[serde(alias = "code_hash")]
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    pub balance: u64,

    /// The minimum gas in order to execute the accumulate
    /// entry-point of the service code (g)
    #[serde(alias = "min_memo_gas")]
    pub accumulate: Gas,

    /// The minimum required for the on transfer entry-point (m)
    #[serde(alias = "min_item_gas")]
    pub transfer: Gas,

    /// The total number of octets used in storage (o)
    #[serde(alias = "bytes")]
    pub total: u64,

    /// The deposit offset of the service account (f)
    #[serde(alias = "deposit_offset")]
    pub offset: u64,

    /// The number of items in storage (i)
    pub items: u32,

    /// The creation time of the service account (r)
    #[serde(alias = "creation_slot")]
    pub creation: u32,

    /// The last update time of the service account (a)
    #[serde(alias = "last_accumulation_slot")]
    pub update: u32,

    /// The parent of the service account (p)
    #[serde(alias = "parent_service")]
    pub parent: u32,
}

impl ServiceInfo {
    /// encode self into the info that host call required
    ///
    /// FIXME: currently just for passing the test, we should
    /// use the account's threshold without conditions.
    pub fn host(&self) -> Result<Vec<u8>> {
        codec::encode(&(
            self.code,
            self.balance,
            self.threshold(),
            self.accumulate,
            self.transfer,
            self.total,
            self.items,
            self.offset,
            self.creation,
            self.update,
            self.parent,
        ))
        .map_err(Into::into)
    }

    /// The threshold of the service account
    pub fn threshold(&self) -> u64 {
        (crate::BALANCE_PER_SERVICE
            + crate::BALANCE_PER_ITEM * self.items as u64
            + crate::BALANCE_PER_OCTET * self.total)
            .saturating_sub(self.offset)
    }
}
