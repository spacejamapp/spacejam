//! Account state

use crate::state::key::{StorageKeyEncode, ACCOUNT_PREIMAGE_PREFIX, ACCOUNT_STORAGE_PREFIX};
use crate::{OpaqueHash, StorageKey};

#[cfg(feature = "crypto")]
pub use crypto_impl::*;

/// C(255, s) - The service account state ((s -> a) δ)
pub fn info(service: u32) -> StorageKey {
    (255, service).key()
}

/// C(s, [(2^32 - 1), k0...28])  ((s ->a ->k ->v) δ)
///
/// from storage dictionary s
pub fn storage(service: u32, k: OpaqueHash) -> StorageKey {
    let mut key = [0u8; 32];
    key[..4].copy_from_slice(&ACCOUNT_STORAGE_PREFIX);
    key[4..].copy_from_slice(&k[..28]);
    (service, key).key()
}

/// C(s, [(2^32 - 2), k0...28]) ((s ->(a ->h) ->p) δ)
pub fn preimage(service: u32, h: OpaqueHash) -> StorageKey {
    let mut key = [0u8; 32];
    key[..4].copy_from_slice(&ACCOUNT_PREIMAGE_PREFIX);
    key[4..].copy_from_slice(&h[1..29]);
    (service, key).key()
}

#[cfg(feature = "crypto")]
mod crypto_impl {
    use super::*;

    /// C(s, [E4(l), H(h)2..30]) (s ->a ->h ->l) δ)
    pub fn lookup(service: u32, lookup: u32, h: OpaqueHash) -> StorageKey {
        let mut key = [0; 32];
        let hashed = crypto::blake2b(&h);
        key[..4].copy_from_slice(&lookup.to_le_bytes());
        key[4..].copy_from_slice(&hashed[2..30]);
        (service, key).key()
    }

    /// Get the diff of the accounts
    pub fn diff(
        accounts: &std::collections::BTreeMap<u32, crate::service::ServiceAccount>,
    ) -> anyhow::Result<Vec<(StorageKey, Vec<u8>)>> {
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
            diff.push((self::info(*index), value));

            // set storage
            for (key, value) in &account.storage {
                let mut buff = [0; 32];
                buff[..key.len()].copy_from_slice(key);
                diff.push((self::storage(*index, buff), value.clone()));
            }

            // set preimage
            for (key, value) in &account.preimage {
                diff.push((self::preimage(*index, *key), value.to_vec()));
            }

            // set lookup
            for ((key, lookup), slots) in &account.lookup {
                diff.push((
                    self::lookup(*index, *lookup, *key),
                    slots
                        .iter()
                        .flat_map(|slot| slot.to_le_bytes())
                        .collect::<Vec<u8>>(),
                ));
            }
        }

        Ok(diff)
    }
}
