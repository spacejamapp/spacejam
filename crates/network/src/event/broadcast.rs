//! Broadcast events

use crate::{stream::ce132, Network};
use score::extrinsic::TicketEnvelope;

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
