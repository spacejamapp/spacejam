//! Preimage handler

use anyhow::Result;
use score::{extrinsic::PreimagesExtrinsic, State, TimeSlot};

/// handle preimage
pub fn handle(
    mut state: State,
    slot: TimeSlot,
    mut preimages: PreimagesExtrinsic,
) -> Result<State> {
    let mut missing = Vec::new();

    // TODO: remove clone
    for (id, acc) in state.accounts.clone().into_iter() {
        for ((hash, _), _) in acc.lookup.clone().into_iter() {
            if !acc.preimage.contains_key(&hash) {
                missing.push((id, hash));
            }
        }
    }

    while let Some(preimage) = preimages.pop() {
        let hash = crypto::blake2b(&preimage.blob);
        if !missing.contains(&(preimage.requester, hash)) {
            anyhow::bail!("Preimage not needed");
        }

        let account = state
            .accounts
            .get_mut(&preimage.requester)
            .ok_or(anyhow::anyhow!("Service account not found"))?;

        let blob = preimage.blob;
        account.preimage.insert(hash, blob.clone());

        let lookup = (hash, blob.len() as u32);

        let slots = account
            .lookup
            .get_mut(&lookup)
            .ok_or(anyhow::anyhow!("Lookup not found"))?;

        slots[2] = slots[1];
        slots[1] = slots[0];
        slots[0] = slot;
    }

    if !preimages.is_empty() {
        anyhow::bail!("Preimages not needed");
    }
    Ok(state)
}
