//! Preimage handler

use anyhow::Result;
use score::{Account, Accounts, TimeSlot, extrinsic::PreimagesExtrinsic};

/// (δ') handle preimage extrinsic
///
/// Validates preimages against post-transfer state and integrates valid ones.
/// Invalid preimages are disregarded without prejudice
///
/// # Arguments
/// * `slot` - Current time slot (τ')
/// * `preimages` - Preimage extrinsic data
/// * `accounts` - Post-transfer account state
pub fn accounts(
    slot: TimeSlot,
    preimages: &PreimagesExtrinsic,
    mut accounts: impl Accounts,
) -> Result<impl Accounts> {
    let mut requester = None;
    for preimage in preimages {
        if let Some(exist) = requester
            && preimage.requester < exist
        {
            anyhow::bail!("Preimages are not ordered");
        }

        // Skip if account doesn't exist (disregard without prejudice)
        requester = Some(preimage.requester);
        let Some(account) = accounts.get(preimage.requester) else {
            anyhow::bail!("Preimage for non-existent account");
        };

        let blob_len = preimage.blob.len() as u32;
        let hash = crypto::blake2b(&preimage.blob);
        tracing::debug!("lookup hash={} len={}", hex::encode(hash), blob_len);
        let Some(slots) = account.lookup(hash, blob_len) else {
            anyhow::bail!("Preimage lookup failed");
        };

        if !slots.is_empty() {
            anyhow::bail!("Preimage already has non-empty lookup slots");
        }

        if account.preimage(hash).is_some() {
            anyhow::bail!("Preimage already exists");
        }

        // Set lookup slots to [τ'] (current time slot)
        let updated_slots = vec![slot];
        account.insert_preimage(hash, preimage.blob.clone());
        account.insert_lookup(hash, blob_len, updated_slots);
    }

    Ok(accounts)
}
