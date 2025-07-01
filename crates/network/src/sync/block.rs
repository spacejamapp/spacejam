//! Block sync implementation

use crate::{peer::Connection, Network};
use score::block::{Head, Header};

impl<C: runtime::Config> Network<C> {
    /// Announce a block to the network
    #[tracing::instrument(skip_all, name = "announce", fields(block = %header.slot, hash = %hex::encode(&header.hash()?[..3])))]
    pub async fn announce(&self, header: Header) -> anyhow::Result<()> {
        let grandpa = self.grandpa().await;
        if let Err(e) = grandpa.accept_local(&header).await {
            tracing::trace!("skip because: {e}");
            return Ok(());
        }

        match self.announce.send(header) {
            Ok(count) => tracing::trace!("broadcasting to {} peers", count),
            Err(e) => tracing::warn!("failed to broadcast block: {e}"),
        }

        Ok(())
    }

    /// Lookup the best head from the network
    pub async fn lookup(&self, best: &Head) -> Vec<Connection> {
        let pool = self.pool.read().await.clone();
        let mut feeds = Vec::new();
        for conn in pool.values() {
            let handshake = conn.handshake.read().await;

            // check if the connection is a feedi
            if handshake.head.hash == best.hash || handshake.leaves.contains(best) {
                feeds.push(conn.clone());
            }
        }

        // we trust the feeds since
        //
        // - they are peers that we've connected to (validators)
        // - the best head is at least a descendant of their finalized heads
        //
        // so we can directly fetch the missing blocks from the feeds.
        feeds.sort_by_key(|conn| conn.rtt());
        feeds
    }
}
