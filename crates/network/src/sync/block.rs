//! Block sync implementation

use crate::{
    peer::Connection,
    stream::{ce128, ext::Read},
    Network,
};
use runtime::{chain::Direction, Handshake};
use score::{
    block::{Head, Header},
    Block,
};

impl<C: runtime::Config> Network<C> {
    /// Announce a block to the network
    #[tracing::instrument(skip_all, name = "announce", fields(block = %header.slot, hash = %hex::encode(&header.hash()?[..3])))]
    pub async fn announce(&self, header: Header) -> anyhow::Result<()> {
        let hash = header.hash()?;
        let slot = header.slot;
        match self.announce.send(header) {
            Ok(count) => tracing::trace!(
                "broadcasting block#{slot}@0x{} to {} peers",
                hex::encode(&hash[..3]),
                count
            ),
            Err(e) => tracing::warn!("failed to broadcast block: {e}"),
        }

        Ok(())
    }

    /// Get the current handshake data
    pub async fn handshake(&self) -> anyhow::Result<Handshake> {
        let chain = self.chain.read().await;
        Ok(chain.grandpa.handshake.clone())
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

    /// Request a block from the network
    pub async fn request(&self, header: &Header) -> anyhow::Result<()> {
        let head = header.head()?;
        let feeds = self.lookup(&head).await;
        for feed in feeds {
            let Ok(mut recv) = ce128::send(
                &feed,
                ce128::Request {
                    hash: header.parent,
                    direction: Direction::Ascending,
                    maximum: 1,
                },
            )
            .await
            .inspect_err(|e| {
                tracing::warn!("failed to send request: {e}, switching to the next peer")
            }) else {
                continue;
            };

            let block = Block::read(&mut recv).await?;
            let hash = block.header.hash()?;
            tracing::trace!(
                "received block#{}@0x{}",
                block.header.slot,
                hex::encode(&hash[..3])
            );

            // check import the block
            let mut chain = self.chain.write().await;
            let imported = chain.import(&block).await?;
            if !imported {
                break;
            }

            // announce the block
            self.announce(block.header.clone()).await?;
        }

        Ok(())
    }
}
