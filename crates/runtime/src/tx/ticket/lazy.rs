//! Lazy verifier

use crypto::vrf::Verifier;
use score::{safrole::ValidatorData, BandersnatchPublic, BandersnatchRingCommitment};
use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;

/// Lazy ring commitment
static LAZY_RING_COMMITMENT: LazyLock<RwLock<BTreeMap<u32, BandersnatchRingCommitment>>> =
    LazyLock::new(|| RwLock::new(BTreeMap::new()));

static LAZY_RING_VERIFIER: LazyLock<RwLock<BTreeMap<u32, Arc<Verifier>>>> =
    LazyLock::new(|| RwLock::new(BTreeMap::new()));

static LAZY_DRAWN_VALIDATORS: LazyLock<RwLock<BTreeMap<u32, Vec<BandersnatchPublic>>>> =
    LazyLock::new(|| RwLock::new(BTreeMap::new()));

/// Only cache last CACHED epochs
const CACHED: usize = 3;

/// Accept drawn validators after accumulation
pub async fn drawn(epoch: u32, drawn: &[ValidatorData]) {
    let keys = drawn.iter().map(|v| v.bandersnatch).collect::<Vec<_>>();
    {
        let mut lazy_drawn = LAZY_DRAWN_VALIDATORS.write().await;
        lazy_drawn.insert(epoch, keys.clone());
        if lazy_drawn.len() > CACHED {
            lazy_drawn.pop_first();
        }
    }

    let _ = tokio::join!(self::commitment(epoch, &keys), self::verifier(epoch, &keys));
}

/// Get the commitment of the next validators at an epoch
///
/// (γ_z') Returns the bandersnatch ring commitment.
pub async fn commitment(epoch: u32, drawn: &Vec<BandersnatchPublic>) -> BandersnatchRingCommitment {
    // check if we have the commitment for the epoch
    {
        let lazy_commitment = LAZY_RING_COMMITMENT.read().await;
        if let Some(commitment) = lazy_commitment.get(&epoch) {
            return *commitment;
        }
    }

    // find if we have the same drawn validators at an epoch
    {
        let mut lazy_commitment = LAZY_RING_COMMITMENT.write().await;
        if let Some(epoch) = self::epoch(drawn).await {
            if let Some(commitment) = lazy_commitment.get(&epoch).cloned() {
                lazy_commitment.insert(epoch, commitment);
                return commitment;
            }
        }
    }

    // if not, create a new commitment
    let commitment = crypto::ring::commitment(drawn);
    let mut lazy_commitment = LAZY_RING_COMMITMENT.write().await;
    lazy_commitment.insert(epoch, commitment);
    if lazy_commitment.len() > CACHED {
        lazy_commitment.pop_first();
    }

    commitment
}

/// Get the verifier of the next validators at an epoch
pub async fn verifier(epoch: u32, drawn: &Vec<BandersnatchPublic>) -> Arc<Verifier> {
    // check if we have the verifier for the epoch
    {
        let lazy_verifier = LAZY_RING_VERIFIER.read().await;
        if let Some(verifier) = lazy_verifier.get(&epoch) {
            return verifier.clone();
        }
    }

    // find if we have the same drawn validators at an epoch
    {
        let mut lazy_verifier = LAZY_RING_VERIFIER.write().await;
        if let Some(epoch) = self::epoch(drawn).await {
            if let Some(verifier) = lazy_verifier.get(&epoch).cloned() {
                lazy_verifier.insert(epoch, verifier.clone());
                return verifier.clone();
            }
        }
    }

    // if not, create a new verifier
    let verifier = Arc::new(crypto::ring::verifier(drawn));
    let mut lazy_verifier = LAZY_RING_VERIFIER.write().await;
    lazy_verifier.insert(epoch, verifier.clone());
    if lazy_verifier.len() > CACHED {
        lazy_verifier.pop_first();
    }

    verifier
}

/// Get the epoch number by previous validators
async fn epoch(drawn: &Vec<BandersnatchPublic>) -> Option<u32> {
    let lazy_drawn = LAZY_DRAWN_VALIDATORS.read().await;
    lazy_drawn
        .iter()
        .find(|(_, keys)| *keys == drawn)
        .map(|(e, _)| *e)
}
