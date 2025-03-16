//! Broadcast events

use crate::{stream::ce132, Network};
use score::{block::Header, extrinsic::TicketEnvelope};

/// Announce a block to the network
#[tracing::instrument(skip_all, name = "announce")]
pub async fn announce<C: score::runtime::Config>(
    runtime: Network<C>,
    header: Box<Header>,
) -> anyhow::Result<()> {
    let grandpa = runtime.grandpa.read().await.clone();
    if let Err(e) = grandpa.accept_local(&header).await {
        tracing::warn!("skip announcing block: {e}");
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
#[tracing::instrument(skip(runtime, ticket), name = "ticket", fields(attempt = %ticket.attempt))]
pub async fn ticket<C: score::runtime::Config>(
    runtime: Network<C>,
    epoch: u32,
    ticket: TicketEnvelope,
) -> anyhow::Result<()> {
    let validators = runtime.grandpa.read().await.grid.curr.clone();
    let pool = runtime.pool.read().await.clone();

    tracing::trace!("broadcasting to {} peers", pool.len());
    for conn in pool.values() {
        let peer: [u8; 32] = conn.address.peer_id.into();
        if validators.iter().any(|v| v.ed25519 == peer) {
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
