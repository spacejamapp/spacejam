//! Lazy verifier

use crypto::vrf::Verifier;
use lru::LruCache;
use score::{BandersnatchPublic, BandersnatchRingCommitment, safrole::ValidatorData};
use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock, Mutex};

static LAZY_RING: LazyLock<Mutex<LruCache<Vec<BandersnatchPublic>, Arc<Verifier>>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(8).unwrap())));

/// Clear all cached data
pub async fn clear() {
    if let Ok(mut map) = LAZY_RING.lock() {
        map.clear();
    }
}

/// Check if the lazy cache is empty
pub fn is_empty() -> bool {
    if let Ok(cache) = LAZY_RING.lock() {
        cache.len() == 0
    } else {
        false
    }
}

/// Accept drawn validators after accumulation
pub fn drawn(drawn: &[ValidatorData]) {
    let keys = drawn.iter().map(|v| v.bandersnatch).collect::<Vec<_>>();
    let _ = self::verifier(&keys);
}

/// Get the commitment of the next validators
///
/// (γ_z') Returns the bandersnatch ring commitment.
pub fn commitment(drawn: &Vec<BandersnatchPublic>) -> BandersnatchRingCommitment {
    self::verifier(drawn).commitment()
}

/// Get the verifier of the next validators
pub fn verifier(drawn: &Vec<BandersnatchPublic>) -> Arc<Verifier> {
    let Ok(mut map) = LAZY_RING.lock() else {
        panic!("failed to lock ring, fix me later");
    };

    if let Some(v) = map.get(drawn) {
        return v.clone();
    }

    drop(map);
    // build new verifier, expensive computation here.
    let verifier = Arc::new(crypto::ring::verifier(drawn));
    let Ok(mut map) = LAZY_RING.lock() else {
        panic!("failed to lock ring, fix me later");
    };

    // double-check after re-acquiring lock
    if let Some(v) = map.get(drawn) {
        return v.clone();
    }

    map.put(drawn.clone(), verifier.clone());
    verifier
}
