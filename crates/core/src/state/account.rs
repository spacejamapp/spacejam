//! Account state

use crate::state::key::{self, StorageKeyEncode, ACCOUNT_PREIMAGE_PREFIX, ACCOUNT_STORAGE_PREFIX};
use crate::{OpaqueHash, TrieKey};

#[cfg(feature = "crypto")]
pub use crypto_impl::*;

/// C(255, s) - The service account state ((s -> a) δ)
pub fn info(service: u32) -> TrieKey {
    (255, service).key()
}

/// C(s, [(2^32 - 2), k0...28]) ((s ->(a ->h) ->p) δ)
pub fn preimage(service: u32, h: OpaqueHash) -> TrieKey {
    let mut key = [0u8; 32];
    key[..4].copy_from_slice(&ACCOUNT_PREIMAGE_PREFIX);
    key[4..].copy_from_slice(&h[1..29]);
    (service, key).key()
}

#[cfg(feature = "crypto")]
mod crypto_impl {
    use super::*;

    /// C(s, [(2^32 - 1), k0...28])  ((s ->a ->k ->v) δ)
    ///
    /// from storage dictionary s
    pub fn storage(service: u32, rkey: &[u8]) -> TrieKey {
        let mut input = service.to_le_bytes().to_vec();
        input.extend_from_slice(rkey);
        let hash = crypto::blake2b(&input);

        // construct the final storage key
        let mut key = [0u8; 31];
        key[..8].copy_from_slice(&key::prefix(service, &ACCOUNT_STORAGE_PREFIX));
        key[8..].copy_from_slice(&hash[..23]);
        key
    }

    /// C(s, [E4(l), H(h)2..30]) (s ->a ->h ->l) δ)
    pub fn lookup(service: u32, lookup: u32, h: OpaqueHash) -> TrieKey {
        let mut key = [0; 32];
        let hashed = crypto::blake2b(&h);
        key[..4].copy_from_slice(&lookup.to_le_bytes());
        key[4..].copy_from_slice(&hashed[2..30]);
        (service, key).key()
    }
}
