//! Service data that not used in the storage
//!
//! Probably just used for testing.

use std::collections::BTreeMap;

use crate::{
    service::{ServiceAccount, ServiceAccountState, ServiceAccountStateJson, ServiceId},
    OpaqueHash,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a service item.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ServiceItem {
    /// The id of the service item
    pub id: ServiceId,

    /// The info of the service item
    #[json(nested)]
    pub data: ServiceAccountData,
}

/// Represents the service account data.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ServiceAccountData {
    /// The service account state
    #[json(nested)]
    pub service: ServiceAccountState,

    /// (a_p) The preimages
    #[serde(default)]
    #[json(nested)]
    pub preimages: Vec<ServicePreimage>,

    /// The storage
    #[serde(default)]
    #[json(nested)]
    pub storage: Vec<ServiceStorage>,
}

impl From<&ServiceAccount> for ServiceAccountData {
    fn from(account: &ServiceAccount) -> Self {
        ServiceAccountData {
            service: account.state(),
            preimages: account
                .preimage
                .iter()
                .map(|(k, v)| ServicePreimage {
                    hash: *k,
                    // TODO: find a better solution for doing this.
                    blob: v.to_vec(),
                })
                .collect(),
            storage: account
                .storage
                .iter()
                .map(|(k, v)| ServiceStorage {
                    key: k.to_vec(),
                    value: v.clone(),
                })
                .collect(),
        }
    }
}

impl From<ServiceAccountData> for ServiceAccount {
    fn from(data: ServiceAccountData) -> Self {
        let mut lookup = BTreeMap::new();
        for preimage in &data.preimages {
            lookup.insert(
                (preimage.hash, preimage.blob.len() as u32),
                Default::default(),
            );
        }

        ServiceAccount {
            storage: data.storage.into_iter().map(|s| (s.key, s.value)).collect(),
            preimage: data
                .preimages
                .into_iter()
                .map(|p| (p.hash, p.blob))
                .collect(),
            lookup,
            code: data.service.code,
            balance: data.service.balance,
            gas: data.service.gas,
        }
    }
}

/// Represents a service preimage.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ServicePreimage {
    /// The hash of the preimage
    #[json(hex)]
    pub hash: OpaqueHash,

    /// The blob of the preimage
    #[json(hex)]
    pub blob: Vec<u8>,
}

/// Represents a service storage.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ServiceStorage {
    /// The key of the storage
    #[json(hex)]
    pub key: Vec<u8>,

    /// The value of the storage
    #[json(hex)]
    pub value: Vec<u8>,
}
