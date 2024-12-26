//! Preimage handler

use anyhow::Result;
use score::{extrinsic::PreimagesExtrinsic, State};

/// handle preimage
pub fn handle(mut state: State, mut preimages: PreimagesExtrinsic) -> Result<State> {
    let mut missing = Vec::new();

    // TODO: remove clone
    for (id, acc) in state.service_accounts.clone().into_iter() {
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

        state
            .service_accounts
            .get_mut(&preimage.requester)
            .ok_or(anyhow::anyhow!("Service account not found"))?
            .preimage
            .insert(hash, preimage.blob);
    }

    if !preimages.is_empty() {
        anyhow::bail!("Preimages not needed");
    }
    Ok(state)
}
