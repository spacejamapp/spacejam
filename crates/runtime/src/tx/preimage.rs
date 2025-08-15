//! Preimage handler

use anyhow::Result;
use score::{
    extrinsic::{Preimage, PreimagesExtrinsic},
    Account, Accounts, TimeSlot,
};
use std::collections::HashSet;

/// (δ') handle preimage extrinsic per Gray Paper eq 331
///
/// Validates preimages against post-transfer state and integrates valid ones.
/// Invalid preimages are disregarded without prejudice per eq 326-332.
///
/// # Arguments
/// * `slot` - Current time slot (τ')
/// * `preimages` - Preimage extrinsic data
/// * `accounts` - Post-transfer account state (accounts_post_xfer)
pub fn accounts(
    slot: TimeSlot,
    preimages: &PreimagesExtrinsic,
    mut accounts: impl Accounts,
) -> Result<impl Accounts> {
    if preimages.windows(2).any(|window| window[0] > window[1]) {
        anyhow::bail!("Preimages are not ordered");
    }

    let length = preimages.len();
    let preimages = preimages.iter().cloned().collect::<HashSet<Preimage>>();
    if preimages.len() != length {
        anyhow::bail!("Preimages contain duplicates");
    }

    // Filter valid preimages per Gray Paper equations 328-332
    // Invalid preimages are "disregarded without prejudice"
    let mut seen_hashes = HashSet::new();
    for preimage in preimages.into_iter() {
        let hash = crypto::blake2b(&preimage.blob);

        // Check for hash collision between different preimages
        if !seen_hashes.insert(hash) {
            anyhow::bail!("Duplicate preimage hash detected");
        }

        // Skip if account doesn't exist (disregard without prejudice)
        let Some(account) = accounts.get(preimage.requester) else {
            anyhow::bail!("Preimage for non-existent account");
        };

        let blob_len = preimage.blob.len() as u32;
        let slots = account.lookup(hash, blob_len).unwrap_or_default();
        if account.preimage(hash).is_some() {
            anyhow::bail!("Preimage already exists");
        }

        if !slots.is_empty() {
            anyhow::bail!("Preimage already has non-empty lookup slots");
        }

        // Set lookup slots to [τ'] (current time slot)
        let updated_slots = vec![slot];
        account.insert_preimage(hash, preimage.blob);
        account.insert_lookup(hash, blob_len, updated_slots);
    }

    Ok(accounts)
}
