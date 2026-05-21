//! Lazy verifier

use crypto::vrf::Verifier;
use lru::LruCache;
use score::{BandersnatchPublic, BandersnatchRingCommitment, safrole::ValidatorData};
use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock, Mutex};

static LAZY_RING: LazyLock<Mutex<LruCache<Vec<BandersnatchPublic>, Arc<Verifier>>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(8).unwrap())));

/// Clear all cached data
pub fn clear() {
    lock().clear();
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
    if let Some(v) = lock().get(drawn).cloned() {
        return v;
    }
    let verifier = Arc::new(crypto::ring::verifier(drawn));
    let mut map = lock();
    if let Some(v) = map.get(drawn) {
        return v.clone();
    }
    map.put(drawn.clone(), verifier.clone());
    verifier
}

fn lock() -> std::sync::MutexGuard<'static, LruCache<Vec<BandersnatchPublic>, Arc<Verifier>>> {
    LAZY_RING.lock().unwrap_or_else(|p| p.into_inner())
}
