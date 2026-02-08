//! Global state root LRU cache keyed by header hash.

use lru::LruCache;
use score::OpaqueHash;
use std::{
    num::NonZeroUsize,
    sync::{LazyLock, Mutex},
};

static CACHE: LazyLock<Mutex<LruCache<OpaqueHash, OpaqueHash>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(16).unwrap())));

/// Get the cached state root for the given header hash.
pub fn get(header_hash: &OpaqueHash) -> Option<OpaqueHash> {
    CACHE.lock().ok()?.get(header_hash).copied()
}

/// Cache a state root for the given header hash.
pub fn set(header_hash: OpaqueHash, state_root: OpaqueHash) {
    if let Ok(mut cache) = CACHE.lock() {
        cache.put(header_hash, state_root);
    }
}
