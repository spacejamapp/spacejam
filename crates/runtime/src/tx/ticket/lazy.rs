//! Lazy verifier

use crypto::vrf::Verifier;
use score::{safrole::ValidatorData, BandersnatchPublic, BandersnatchRingCommitment};
use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock},
};
use tokio::sync::Mutex;

static LAZY_RING: LazyLock<Mutex<BTreeMap<u32, Arc<Verifier>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

/// Only cache last CACHED epochs
const CACHED: usize = 6;

/// Clear all cached data
pub async fn clear() {
    LAZY_RING.lock().await.clear();
}

/// Check if the lazy cache is empty
pub async fn is_empty() -> bool {
    LAZY_RING.lock().await.is_empty()
}

/// Accept drawn validators after accumulation
pub async fn drawn(epoch: u32, drawn: &[ValidatorData]) {
    let keys = drawn.iter().map(|v| v.bandersnatch).collect::<Vec<_>>();
    let _ = self::verifier(epoch, &keys).await;
}

/// Get the commitment of the next validators at an epoch
///
/// (γ_z') Returns the bandersnatch ring commitment.
pub async fn commitment(epoch: u32, drawn: &Vec<BandersnatchPublic>) -> BandersnatchRingCommitment {
    self::verifier(epoch, drawn).await.commitment()
}

/// Get the verifier of the next validators at an epoch
pub async fn verifier(epoch: u32, drawn: &Vec<BandersnatchPublic>) -> Arc<Verifier> {
    // check if we have the verifier for the epoch
    {
        let lazy_verifier = LAZY_RING.lock().await;
        if let Some(verifier) = lazy_verifier.get(&epoch) {
            return verifier.clone();
        }
    }

    // find if we have the same drawn validators at an epoch
    {
        let mut lazy_verifier = LAZY_RING.lock().await;
        for verifier in lazy_verifier.clone().values() {
            if verifier.ring() != *drawn {
                continue;
            }

            lazy_verifier.insert(epoch, verifier.clone());
            return verifier.clone();
        }
    }

    // create a new verifier
    let mut lazy_verifier = LAZY_RING.lock().await;
    let drawn = drawn.clone();
    let verifier = Arc::new(
        tokio::task::spawn_blocking(move || crypto::ring::verifier(&drawn))
            .await
            .expect("Failed to create verifier"),
    );

    // Reacquire write lock to insert the result
    lazy_verifier.insert(epoch, verifier.clone());
    if lazy_verifier.len() > CACHED {
        lazy_verifier.pop_first();
    }

    verifier
}
