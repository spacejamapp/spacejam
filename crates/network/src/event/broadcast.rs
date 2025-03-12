//! Broadcast events

use crate::{stream::ce132, Network};
use score::{block::Header, extrinsic::TicketEnvelope};

/// Announce a block to the network
pub async fn announce<C: score::runtime::Config>(
    runtime: Network<C>,
    header: Box<Header>,
) -> anyhow::Result<()> {
    let grandpa = runtime.grandpa.read().await.clone();
    if let Err(e) = grandpa.verify(&header).await {
        tracing::warn!("{e}");
        return Ok(());
    }

    // broadcast the block to the network
    match runtime.announce.send(*header) {
        Ok(count) => tracing::trace!("broadcasting block to {} peers", count),
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
    let pool = runtime.pool.read().await.clone();
    for conn in pool.values() {
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
