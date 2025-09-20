//! Lazy verifier

use crypto::vrf::Verifier;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use score::{BandersnatchPublic, BandersnatchRingCommitment, safrole::ValidatorData};
use std::sync::Arc;

static LAZY_RING: Lazy<DashMap<u32, Arc<Verifier>>> = Lazy::new(DashMap::new);

/// Only cache last CACHED epochs
const CACHED: usize = 6;

/// Clear all cached data
pub async fn clear() {
    LAZY_RING.clear();
}

/// Check if the lazy cache is empty
pub fn is_empty() -> bool {
    LAZY_RING.is_empty()
}

/// Accept drawn validators after accumulation
pub fn drawn(epoch: u32, drawn: &[ValidatorData]) {
    let keys = drawn.iter().map(|v| v.bandersnatch).collect::<Vec<_>>();
    let _ = self::verifier(epoch, &keys);
}

/// Get the commitment of the next validators at an epoch
///
/// (γ_z') Returns the bandersnatch ring commitment.
pub fn commitment(epoch: u32, drawn: &Vec<BandersnatchPublic>) -> BandersnatchRingCommitment {
    self::verifier(epoch, drawn).commitment()
}

/// Get the verifier of the next validators at an epoch
pub fn verifier(epoch: u32, drawn: &Vec<BandersnatchPublic>) -> Arc<Verifier> {
    if let Some(entry) = LAZY_RING.get(&epoch) {
        return entry.clone();
    }

    if let Some(entry) = LAZY_RING.iter().find(|e| e.value().ring() == *drawn) {
        let v = entry.value().clone();
        drop(entry);
        LAZY_RING.insert(epoch, v.clone());
        return v;
    }

    let verifier = Arc::new(crypto::ring::verifier(drawn));
    LAZY_RING.insert(epoch, verifier.clone());
    if LAZY_RING.len() > CACHED
        && let Some(oldest) = LAZY_RING.iter().map(|e| *e.key()).min()
    {
        LAZY_RING.remove(&oldest);
    }

    verifier
}
