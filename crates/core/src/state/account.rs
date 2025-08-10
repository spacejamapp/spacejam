//! Account state

use crate::state::key::{self, StorageKeyEncode};
use crate::TrieKey;

/// C(255, s) - The service account state ((s -> a) δ)
pub fn info(service: u32) -> TrieKey {
    (255, service).key()
}

#[cfg(feature = "crypto")]
/// compose general service data
pub fn data(service: u32, prefix: [u8; 4], key: &[u8]) -> TrieKey {
    let mut hashed = prefix.to_vec();
    hashed.extend_from_slice(&key);
    let hash = crypto::blake2b(&hashed);
    (service, hash).key()
}

#[cfg(not(feature = "crypto"))]
pub fn data(_service: u32, _prefix: [u8; 4], _key: &[u8]) -> TrieKey {
    unimplemented!("please enable feature `crypto`")
}

/// Service storage key construction
///
/// C(s, E4(2^32 - 1) ++ k) -> v
pub fn storage(service: u32, key: &[u8]) -> TrieKey {
    self::data(service, key::ACCOUNT_STORAGE_PREFIX, key)
}

/// Service preimage key construction
///
/// C(s, E4(2^32 - 2) ++ h) -> p
pub fn preimage(service: u32, key: [u8; 32]) -> TrieKey {
    self::data(service, key::ACCOUNT_PREIMAGE_PREFIX, &key)
}

/// Service lookup key construction
///
/// C(s, E4(l) ++ h) -> slots
pub fn lookup(service: u32, length: u32, key: [u8; 32]) -> TrieKey {
    self::data(service, length.to_le_bytes(), &key)
}
