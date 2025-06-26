//! Block announcement stream.

use crate::{
    peer::PeerId,
    stream::ext::{Read, Write},
    Network,
};
use anyhow::Context;
use quinn::{RecvStream, SendStream};
use runtime::Handshake;

mod announce;

impl<C: runtime::Config> Network<C> {
    /// Send a block announcement.
    pub async fn send_up0(&self, peer: PeerId) -> anyhow::Result<()> {
        let conn = self.get_conn(peer).await?;
        let (mut send, mut recv) = conn.open_bi().await.context("failed to open bi-stream")?;

        // 1. send the stream type
        send.write(&[0]).await?;

        // 2. wait for the handshake
        let mut buf = [0; 4];
        recv.read_exact(&mut buf).await?;
        let length = u32::from_le_bytes(buf);
        let mut buf = vec![0; length as usize];
        recv.read_exact(&mut buf).await?;
        let handshake: Handshake = codec::decode(&buf)?;
        conn.handshake.write().await.head = handshake.head;

        // 3. send the handshake
        let grandpa = self.grandpa().await;
        let encoded = codec::encode(&grandpa.handshake)?;
        let length = encoded.len() as u32;
        send.write(&length.to_le_bytes()).await?;
        send.write(&encoded).await?;

        // 4. announcement loop
        let runtime = self.clone();
        tokio::spawn(async move {
            if let Err(e) = announce::spawn(runtime.clone(), send, recv, conn).await {
                tracing::error!("failed to spawn announcement loop: {e:?}");
            }
        });

        Ok(())
    }

    /// Receive a block announcement
    pub async fn recv_up0(
        &self,
        peer: PeerId,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        tracing::debug!("receiving up0 stream from {peer}");
        let conn = self.get_conn(peer).await?;

        // 1. send and receive the handshake data.
        let (hsend, hrecv): (Result<(), anyhow::Error>, Result<(), anyhow::Error>) = tokio::join!(
            async {
                let grandpa = self.grandpa().await;
                let mut handshake = grandpa.handshake.clone();
                handshake.leaves.insert(handshake.head.clone());

                handshake
                    .write(&mut send)
                    .await
                    .context("failed to send handshake")
            },
            async {
                let handshake = Handshake::read(&mut recv)
                    .await
                    .context("failed to read handshake")?;
                conn.handshake.write().await.head = handshake.head;
                Ok(())
            }
        );

        hsend?;
        hrecv?;

        // 3. announcement loop.
        let runtime = self.clone();
        tokio::spawn(async move {
            if let Err(e) = announce::spawn(runtime.clone(), send, recv, conn).await {
                tracing::error!("failed to spawn announcement loop: {e:?}");
            }
        });

        Ok(())
    }
}
