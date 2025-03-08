//! Broadcast events

use crate::{stream::ce132, Network};
use score::{block::Header, extrinsic::TicketEnvelope, runtime::Head};

/// Announce a block to the network
pub async fn announce<C: score::runtime::Config>(
    runtime: Network<C>,
    header: Box<Header>,
    head: Head,
) -> anyhow::Result<()> {
    tracing::info!(
        "announcing block#{}@0x{}",
        header.slot,
        hex::encode(header.hash()?)
    );
    if let Err(e) = runtime.grandpa.read().await.verify(&header).await {
        tracing::trace!(
            "block#{}@0x{} verification failed: {e}",
            header.slot,
            hex::encode(header.hash()?)
        );
        return Ok(());
    }

    // broadcast the block to the network
    match runtime.announce.send((*header, head)) {
        Ok(count) => tracing::trace!("broadcasted block to {} peers", count),
        Err(e) => tracing::warn!("failed to broadcast block: {e}"),
    }

    Ok(())
}

/// Broadcast a ticket to all current validators in the network.
pub async fn ticket<C: score::runtime::Config>(
    runtime: Network<C>,
    epoch: u32,
    ticket: TicketEnvelope,
) -> anyhow::Result<()> {
    let validators = runtime.grandpa.read().await.grid.curr;
    for conn in runtime.pool.read().await.values() {
        let peer: [u8; 32] = conn.address.peer_id.into();
        if validators.contains(&peer) {
            let (send, recv) = conn.open_bi().await?;
            ce132::send(
                send,
                recv,
                ce132::Request {
                    epoch,
                    ticket: ticket.clone(),
                },
            )
            .await?;
        }
    }

    Ok(())
}
