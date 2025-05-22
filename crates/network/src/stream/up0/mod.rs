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

        // 1. send the handshake
        let grandpa = self.grandpa.read().await;
        let handshake = grandpa.handshake.clone();

        // 2. write the handshake message
        handshake.write(&mut send, Some(0)).await?;

        // 3. get the handshake from remote
        let handshake = self::handshake(&mut recv).await?;
        conn.handshake.write().await.head = handshake.head;

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

/// Read the handshake from the stream.
async fn handshake(recv: &mut RecvStream) -> anyhow::Result<Handshake> {
    let mut buf = [0; 4];
    recv.read_exact(&mut buf).await?;
    let length = u32::from_le_bytes(buf);

    let mut buf = vec![0; length as usize];
    recv.read_exact(&mut buf).await?;

    let handshake = codec::decode(&buf)?;
    Ok(handshake)
}
