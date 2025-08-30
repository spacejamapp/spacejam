//! Lazy verifier

use crate::tx::ticket::Error;
use crypto::vrf::Verifier;
use score::{safrole::ValidatorData, BandersnatchPublic, BandersnatchRingCommitment};
use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock},
    time::Instant,
};
use tokio::sync::{Mutex, RwLock};

/// Lazy ring commitment
static LAZY_RING_COMMITMENT: LazyLock<Mutex<BTreeMap<u32, BandersnatchRingCommitment>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

static LAZY_RING_VERIFIER: LazyLock<Mutex<BTreeMap<u32, Arc<Verifier>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

static LAZY_DRAWN_VALIDATORS: LazyLock<RwLock<BTreeMap<u32, Vec<BandersnatchPublic>>>> =
    LazyLock::new(|| RwLock::new(BTreeMap::new()));

/// Only cache last CACHED epochs
const CACHED: usize = 3;

/// Accept drawn validators after accumulation
pub fn drawn(epoch: u32, drawn: &[ValidatorData]) {
    let keys = drawn.iter().map(|v| v.bandersnatch).collect::<Vec<_>>();
    tokio::spawn(async move {
        let mut lazy_drawn = LAZY_DRAWN_VALIDATORS.write().await;
        lazy_drawn.insert(epoch, keys.clone());
        if lazy_drawn.len() > CACHED {
            lazy_drawn.pop_first();
        }

        let now = Instant::now();
        let _ = tokio::join!(self::commitment(epoch, &keys), self::verifier(epoch, &keys));
        tracing::info!(
            "ring commitment and verifier time: {}ms",
            now.elapsed().as_millis()
        );
    });
}

/// Get the commitment of the next validators at an epoch
///
/// (γ_z') Returns the bandersnatch ring commitment.
pub async fn commitment(
    epoch: u32,
    drawn: &Vec<BandersnatchPublic>,
) -> Result<BandersnatchRingCommitment, Error> {
    let now = Instant::now();
    let mut lazy_commitment = LAZY_RING_COMMITMENT.lock().await;
    if let Some(commitment) = lazy_commitment.get(&epoch) {
        return Ok(commitment.clone());
    }

    // find if we have the same drawn validators at an epoch
    if let Some(epoch) = self::epoch(drawn).await {
        if let Some(commitment) = lazy_commitment.get(&epoch).cloned() {
            lazy_commitment.insert(epoch, commitment.clone());
            return Ok(commitment.clone());
        }
    }

    // if not, create a new commitment
    let drawn = drawn.clone();
    let commitment = tokio::task::spawn_blocking(move || crypto::ring::commitment(drawn))
        .await
        .map_err(|_| Error::Reserved)?;
    lazy_commitment.insert(epoch, commitment.clone());
    if lazy_commitment.len() > CACHED {
        lazy_commitment.pop_first();
    }

    tracing::info!("ring commitment time: {}ms", now.elapsed().as_millis());
    return Ok(commitment);
}

/// Get the verifier of the next validators at an epoch
pub async fn verifier(epoch: u32, drawn: &Vec<BandersnatchPublic>) -> Result<Arc<Verifier>, Error> {
    let now = Instant::now();
    let mut lazy_verifier = LAZY_RING_VERIFIER.lock().await;
    if let Some(verifier) = lazy_verifier.get(&epoch) {
        return Ok(verifier.clone());
    }

    // find if we have the same drawn validators at an epoch
    if let Some(epoch) = self::epoch(drawn).await {
        if let Some(verifier) = lazy_verifier.get(&epoch) {
            return Ok(verifier.clone());
        }
    }

    // if not, create a new verifier
    let drawn = drawn.clone();
    let verifier = Arc::new(
        tokio::task::spawn_blocking(move || crypto::ring::verifier(drawn))
            .await
            .map_err(|_| Error::Reserved)?,
    );
    lazy_verifier.insert(epoch, verifier.clone());
    if lazy_verifier.len() > CACHED {
        lazy_verifier.pop_first();
    }

    tracing::info!("ring verifier time: {}ms", now.elapsed().as_millis());
    return Ok(verifier);
}

/// Get the epoch number by previous validators
async fn epoch(drawn: &Vec<BandersnatchPublic>) -> Option<u32> {
    let lazy_drawn = LAZY_DRAWN_VALIDATORS.read().await;
    lazy_drawn
        .iter()
        .find(|(_, keys)| *keys == drawn)
        .map(|(e, _)| *e)
}
