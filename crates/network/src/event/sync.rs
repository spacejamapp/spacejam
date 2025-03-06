//! Handler of sync events

use crate::Network;
use score::{block::Header, runtime::Head, TimeSlot};

/// Announce a block to the network
pub async fn announce<C: score::runtime::Config>(
    runtime: Network<C>,
    header: Box<Header>,
    head: Head,
) -> anyhow::Result<()> {
    if let Err(e) = runtime.grandpa.read().await.verify(&header).await {
        anyhow::bail!(e);
    }

    // broadcast the block to the network
    runtime.announce.send((*header, head))?;
    Ok(())
}

/// Select the best chain.
pub async fn select_best_chain<C: score::runtime::Config>(
    runtime: Network<C>,
    slot: TimeSlot,
) -> anyhow::Result<()> {
    let grandpa = runtime.grandpa.read().await;
    if slot <= grandpa.head.slot {
        return Ok(());
    }

    // select the best head from the grandpa.
    let Some(_best) = grandpa.select_best_head() else {
        return Ok(());
    };

    // shall we request the missing blocks here?
    //
    // 1. we get the best head from grandpa.
    // 2. according to the author of best head, we get the address of the peer.?
    // 3. once we got the address of the peer, we create a new connection to the peer.
    // 4. request the missing blocks from the peer.
    // 5. finliaze the best chain.
    // 6. on finalizing blocks, we update the grandpa state.

    Ok(())
}
