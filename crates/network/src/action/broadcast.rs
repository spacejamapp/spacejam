//! Broadcast events

use crate::{stream::ce132, Network};
use score::{block::Header, extrinsic::TicketEnvelope};

impl<C: runtime::Config> Network<C> {
    /// Announce a block to the network
    #[tracing::instrument(skip_all, name = "announce", fields(block = %header.slot, hash = %hex::encode(&header.hash()?[..3])))]
    pub async fn announce(&self, header: Box<Header>) -> anyhow::Result<()> {
        let grandpa = self.grandpa.read().await.clone();
        if let Err(e) = grandpa.accept_local(&header).await {
            tracing::warn!("skip because: {e}");
            return Ok(());
        }

        // broadcast the block to the network
        if header.slot > grandpa.handshake.head.slot {
            self.select_best_chain(header.slot).await?;
        } else {
            tracing::trace!(
                "skipping best chain selection: incoming#{}, grandpa#{}",
                header.slot,
                grandpa.handshake.head.slot
            );
        }

        match self.announce.send(*header) {
            Ok(count) => tracing::trace!("broadcasting to {} peers", count),
            Err(e) => tracing::warn!("failed to broadcast block: {e}"),
        }

        Ok(())
    }

    /// Broadcast a ticket to all current validators in the network.
    #[tracing::instrument(skip_all, name = "ticket", fields(attempt = %ticket.attempt))]
    pub async fn ticket(&self, epoch: u32, ticket: TicketEnvelope) -> anyhow::Result<()> {
        let validators = self.grandpa.read().await.grid.curr.clone();
        let pool = self.pool.read().await.clone();

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
}
