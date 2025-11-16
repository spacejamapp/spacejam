//! Lazy verifier

use crypto::vrf::Verifier;
use once_cell::sync::Lazy;
use score::{BandersnatchPublic, BandersnatchRingCommitment, safrole::ValidatorData};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

static LAZY_RING: Lazy<Mutex<BTreeMap<u32, Arc<Verifier>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

/// Only cache last CACHED epochs
const CACHED: usize = 6;

/// Clear all cached data
pub async fn clear() {
    if let Ok(mut map) = LAZY_RING.lock() {
        map.clear();
    }
}

/// Check if the lazy cache is empty
pub fn is_empty() -> bool {
    if let Ok(map) = LAZY_RING.lock() {
        map.is_empty()
    } else {
        false
    }
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
    if let Some(v) = LAZY_RING
        .lock()
        .map(|map| map.get(&epoch).cloned())
        .ok()
        .flatten()
    {
        return v.clone();
    }

    // same validator set already cached
    if let Ok(mut map) = LAZY_RING.lock()
        && let Some(v) = map.values().find(|v| v.ring() == *drawn)
    {
        let v = v.clone();
        map.insert(epoch, v.clone());
        return v;
    }

    // build new verifier
    let verifier = Arc::new(crypto::ring::verifier(drawn));
    let Ok(mut map) = LAZY_RING.lock() else {
        panic!("failed to lock ring, fix me later");
    };
    map.insert(epoch, verifier.clone());
    while map.len() > CACHED {
        map.pop_first();
    }
    verifier
}
