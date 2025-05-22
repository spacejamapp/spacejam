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
        let grandpa = self.grandpa.read().await;
        let encoded = codec::encode(&grandpa.handshake)?;
        let length = encoded.len() as u32;
        send.write(&length.to_le_bytes()).await?;
        send.write(&encoded).await?;
        tracing::trace!("up0 established");

        // 4. announcement loop
        let runtime = self.clone();
        tokio::spawn(async move {
            announce::unchecked(runtime.clone(), send, recv, conn).await;
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
        let conn = self.get_conn(peer).await?;

        // 1. read the grandpa data
        let handshake = Handshake::read(&mut recv).await?;
        conn.handshake.write().await.head = handshake.head;

        // 2. send the handshake data.
        let grandpa = self.grandpa.read().await;
        grandpa.handshake.write(&mut send, None).await?;

        // 3. announcement loop.
        let runtime = self.clone();
        tokio::spawn(async move {
            announce::unchecked(runtime.clone(), send, recv, conn).await;
        });

        Ok(())
    }
}
