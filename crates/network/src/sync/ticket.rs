//! Broadcast events

use crate::{
    peer::PeerId,
    stream::{ce131, ce132},
    Connection, Network,
};
use anyhow::Context;
use score::extrinsic::{Ticket, TicketEnvelope};

impl<C: runtime::Config> Network<C> {
    /// Broadcast a ticket to validators in the network.
    #[tracing::instrument(skip_all, name = "ticket", fields(attempt = %ticket.envelope.attempt))]
    pub async fn submit(&self, epoch: u32, ticket: Ticket) -> anyhow::Result<()> {
        let validators = self.runtime.grid().await.next;
        let pool = self.pool.read().await.clone();
        let validator: PeerId = validators[ticket.submission()].ed25519.into();
        let conn = pool
            .get(&validator)
            .ok_or_else(|| anyhow::anyhow!("validator not found: {validator:?}"))?;

        let (send, recv) = conn.open_bi().await.context(format!(
            "failed to open bi-stream to submit ticket to {validator}"
        ))?;

        tracing::trace!(
            "submitting ticket#{} to {validator}",
            ticket.envelope.attempt
        );
        ce131::send(
            send,
            recv,
            ce132::Request {
                epoch,
                ticket: ticket.envelope,
            },
        )
        .await
    }

    /// subscribe tickets to the network
    pub async fn subscribe_tickets(&self) -> anyhow::Result<()> {
        let pool = self.pool.read().await.clone();
        let tickets = self.runtime.tickets.lock().await.clone();
        if tickets.is_empty() {
            return Ok(());
        }

        let validators = self.runtime.grid().await.next;
        let me = self.me();
        let this = validators
            .iter()
            .find(|v| v.bandersnatch == me)
            .map(|v| v.ed25519.into())
            .ok_or_else(|| anyhow::anyhow!("not in the validator list"))?;
        for (peer, conn) in pool {
            if peer == this {
                continue;
            }

            if let Err(e) = self.send_tickets(conn, tickets.clone()).await {
                tracing::warn!("failed to send tickets to {peer}: {e}");
            }
        }

        self.runtime.tickets.lock().await.clear();
        Ok(())
    }

    async fn send_tickets(
        &self,
        conn: Connection,
        tickets: Vec<(u32, TicketEnvelope)>,
    ) -> anyhow::Result<()> {
        for (epoch, ticket) in tickets.clone() {
            let (send, recv) = conn
                .open_bi()
                .await
                .map_err(|e| anyhow::anyhow!("failed to open bi-stream: {e}"))?;

            ce132::send(send, recv, ce132::Request { epoch, ticket }).await?;
        }

        Ok(())
    }
}
