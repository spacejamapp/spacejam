//! Block sync implementation

use crate::{
    peer::Connection,
    stream::{ce128, ext::Read},
    Network,
};
use runtime::chain::Direction;
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
    ///
    /// returns true if imported successfully
    pub async fn request(
        &self,
        conn: &Connection,
        header: &Header,
        direction: Direction,
    ) -> anyhow::Result<(bool, Header)> {
        let mut recv = ce128::send(
            conn,
            match direction {
                Direction::Ascending => ce128::Request {
                    hash: header.parent,
                    direction: Direction::Ascending,
                    maximum: 1,
                },
                Direction::Descending => ce128::Request {
                    hash: header.parent,
                    direction: Direction::Descending,
                    maximum: 1,
                },
            },
        )
        .await?;

        let block = Block::read(&mut recv).await?;
        let hash = block.header.hash()?;
        tracing::trace!(
            "received block#{}@0x{}",
            block.header.slot,
            hex::encode(&hash[..3])
        );

        // check import the block
        let imported = self.import(&block).await?;
        if imported {
            self.announce(block.header.clone()).await?;
        }

        Ok((imported, block.header))
    }
}
