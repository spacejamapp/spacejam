//! Preimage extrinsic handler

use account::{Account, Accounts};
use anyhow::Result;
use score::{TimeSlot, extrinsic::PreimagesExtrinsic};

/// Validate preimages against the prior state
pub fn validate<A: Accounts>(accounts: &mut A, preimages: &PreimagesExtrinsic) -> Result<()> {
    let mut prev: Option<&score::extrinsic::Preimage> = None;
    for preimage in preimages {
        if let Some(p) = prev
            && preimage <= p
        {
            anyhow::bail!("preimages not sorted or unique");
        }
        prev = Some(preimage);

        let hash = crypto::blake2b(&preimage.blob);
        let len = preimage.blob.len() as u32;
        if !accounts.is_providable(preimage.requester, hash, len) {
            anyhow::bail!("preimage not required");
        }
    }
    Ok(())
}

/// (δ') Integrate providable preimages into the post-transfer state
pub fn accounts<A: Accounts>(
    slot: TimeSlot,
    preimages: &PreimagesExtrinsic,
    mut accounts: A,
) -> A {
    for preimage in preimages {
        let hash = crypto::blake2b(&preimage.blob);
        let len = preimage.blob.len() as u32;
        if !accounts.is_providable(preimage.requester, hash, len) {
            continue;
        }
        let account = accounts.get(preimage.requester).expect("just checked");
        account.insert_preimage(hash, preimage.blob.clone());
        account.insert_lookup(hash, len, vec![slot]);
    }
    accounts
}
