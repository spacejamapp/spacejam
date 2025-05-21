//! Block announcement stream.

use crate::{peer::PeerId, Network};
use anyhow::Context;
use quinn::{RecvStream, SendStream};
use runtime::{Handshake, Runtime};
use score::{
    block::{Head, Header},
    OpaqueHash, TimeSlot,
};
use std::sync::Arc;
use tokio::sync::RwLock;

mod announce;

impl<C: runtime::Config> Network<C> {
    /// Send a block announcement.
    pub async fn send_up0(&self, peer: PeerId) -> anyhow::Result<()> {
        let conn = self.get_conn(peer).await?;
        let (mut send, mut recv) = conn.open_bi().await.context("failed to open bi-stream")?;

        // 1. send the handshake
        let grandpa = self.grandpa.read().await;
        let handshake = grandpa.handshake.clone();
        let mut buf = vec![0];
        buf.extend_from_slice(&codec::encode(&handshake)?);
        send.write(&buf).await?;

        // 2. get the handshake from remote
        let handshake = self::handshake(&mut recv).await?;
        conn.handshake.write().await.head = handshake.head;

        // 3. announcement loop
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
        let handshake = self::handshake(&mut recv).await?;
        conn.handshake.write().await.head = handshake.head;

        // 2. send the handshake data.
        let grandpa = self.grandpa.read().await;
        send.write(&codec::encode(&grandpa.handshake)?).await?;

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
    let mut buf = vec![];
    while let Ok(Some(chunk)) = recv.read_chunk(1, true).await {
        buf.extend_from_slice(&chunk.bytes);
        if let Ok(handshake) = codec::decode(&buf) {
            return Ok(handshake);
        }
    }

    anyhow::bail!("failed to read handshake");
}
