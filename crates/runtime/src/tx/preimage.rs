//! Preimage handler

use anyhow::Result;
use score::{
    Account, Accounts, TimeSlot,
    extrinsic::{Preimage, PreimagesExtrinsic},
};
use std::collections::HashSet;

/// (δ') handle preimage
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

    // transit preimages
    for preimage in preimages.into_iter() {
        let account = accounts
            .get(preimage.requester)
            .ok_or(anyhow::anyhow!("Account not found"))?;

        let hash = crypto::blake2b(&preimage.blob);
        let exist = account.preimage(hash).is_some();
        let blob_len = preimage.blob.len() as u32;
        let mut slots = account.lookup(hash, blob_len).unwrap_or_default();

        // The data must have been solicited by a service but
        // not yet provided in the prior state.
        //
        // FIXME: The formula in graypaper seems mismatched with
        // the preimage tests and the trace tests.
        if exist || !slots.is_empty() {
            anyhow::bail!("Preimage not needed");
        }

        if slots.len() >= 3 {
            slots.resize(3, 0);
            slots[2] = slots[1];
            slots[1] = slots[0];
            slots[0] = slot;
        } else {
            slots.push(slot);
        }

        account.insert_preimage(hash, preimage.blob);
        account.insert_lookup(hash, blob_len, slots);
    }

    Ok(accounts)
}
