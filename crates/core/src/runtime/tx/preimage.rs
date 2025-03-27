//! Preimage handler

use crate::{
    extrinsic::{Preimage, PreimagesExtrinsic},
    service::ServiceAccount,
    TimeSlot,
};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};

/// handle preimage
pub fn accounts(
    slot: TimeSlot,
    preimages: &PreimagesExtrinsic,
    accounts: &BTreeMap<u32, ServiceAccount>,
) -> Result<BTreeMap<u32, ServiceAccount>> {
    let mut missing = Vec::new();
    for (id, acc) in accounts.iter() {
        for ((hash, _), _) in acc.lookup.iter() {
            if !acc.preimage.contains_key(hash) {
                missing.push((id, hash));
            }
        }
    }

    // The lookup the validator. extrinsic is a sequence of pairs of
    // service indices and data.
    //
    // The objective statistics are updated in line with their
    // These pairs must be ordered and without duplicates (equa-
    // tion 12.35 requires this).

    // check ordering
    if preimages.windows(2).any(|window| window[0] > window[1]) {
        anyhow::bail!("Preimages are not ordered");
    }

    // check for duplicates
    let spreimages = preimages.iter().cloned().collect::<HashSet<Preimage>>();

    if spreimages.len() != preimages.len() {
        anyhow::bail!("Preimages contain duplicates");
    }

    // transit preimages
    let mut next = accounts.clone();
    let mut preimages = preimages.clone();
    while let Some(preimage) = preimages.pop() {
        let hash = crypto::blake2b(&preimage.blob);
        if !missing.contains(&(&preimage.requester, &hash)) {
            anyhow::bail!("Preimage not needed");
        }

        let account = next
            .get_mut(&preimage.requester)
            .ok_or(anyhow::anyhow!("Service account not found"))?;

        let blob = preimage.blob.clone();
        let lookup = (hash, blob.len() as u32);
        account.preimage.insert(hash, blob);

        let slots = account
            .lookup
            .get_mut(&lookup)
            .ok_or(anyhow::anyhow!("Lookup not found"))?;

        slots[2] = slots[1];
        slots[1] = slots[0];
        slots[0] = slot;
    }

    if !preimages.is_empty() {
        anyhow::bail!("Preimages not empty");
    }
    Ok(next)
}
