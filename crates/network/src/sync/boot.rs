//! Bootstrap the network if bootnode is specified.

use std::time::Duration;

use anyhow::Context;
use quinn::{Connection, VarInt};
use runtime::{chain::Direction, Handshake};
use score::Block;

use crate::{
    stream::{
        ce128,
        ext::{Read, Write},
    },
    Network,
};

impl<C: runtime::Config> Network<C> {
    /// Bootstrap the network if bootnode is specified.
    pub async fn bootstrap(&self) -> anyhow::Result<()> {
        let Some(bootnode) = &self.bootnode else {
            return Ok(());
        };

        tracing::info!("bootstrapping with the bootnode: {}", bootnode);
        // 1. dial the bootnode
        let conn = self
            .transport
            .connect(bootnode.address, bootnode.peer_id.to_string().as_str())?
            .await?;

        let handshake = self.fetch_handshake(&conn).await?;
        let finalized = self.runtime.finalized().await;
        tracing::info!(
            "finalized: #{} remote: #{}",
            finalized.slot,
            handshake.head.slot,
        );

        let mut count = 0;
        let mut current = finalized.hash;
        while current != handshake.head.hash {
            let request = ce128::Request {
                hash: current,
                direction: Direction::Ascending,
                maximum: 1,
            };

            let (mut send, mut recv) = conn.open_bi().await?;
            send.write(&[128]).await?;
            request.write(&mut send).await?;
            send.finish()?;

            let block = Block::read(&mut recv).await?;
            let hash = block.header.hash()?;
            if hash == handshake.head.hash {
                break;
            }

            self.import(&block).await?;
            self.finalize().await?;
            current = hash;
            count += 1;

            if count % 100 == 0 {
                tracing::info!("synced {} blocks", count);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }

        tracing::info!("node synced!");
        Ok(())
    }

    /// NOTE: this method should only being used in the light node!
    async fn fetch_handshake(&self, conn: &Connection) -> anyhow::Result<Handshake> {
        let (mut send, mut recv) = conn.open_bi().await.context("failed to open bi-stream")?;
        send.write(&[0]).await?;
        let (hsend, hrecv): (Result<(), anyhow::Error>, Result<Handshake, anyhow::Error>) = tokio::join!(
            async {
                let handshake = self.handshake().await;
                handshake
                    .write(&mut send)
                    .await
                    .context("failed to send handshake")
            },
            async {
                let handshake = Handshake::read(&mut recv)
                    .await
                    .context("failed to read handshake")?;
                Ok(handshake)
            }
        );

        send.finish()?;
        recv.stop(VarInt::from(0_u8))?;
        hsend?;
        hrecv
    }
}
