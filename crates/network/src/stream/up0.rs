//! Block announcement stream.

use crate::{peer::Manager, Context};
use quinn::{RecvStream, SendStream};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Send a block announcement.
pub async fn send<C: Context>(
    send: SendStream,
    recv: RecvStream,
    context: Arc<C>,
    manager: Arc<RwLock<Manager>>,
) -> anyhow::Result<()> {
    Ok(())
}

/// Receive a block announcement
pub async fn recv<C: Context>(
    mut send: SendStream,
    recv: RecvStream,
    context: Arc<C>,
    manager: Arc<RwLock<Manager>>,
) -> anyhow::Result<()> {
    let mut brx = manager.read().await.btx.subscribe();

    // handshake
    // let data = context.up0_handshake().await?;

    // announcement loop.
    tokio::spawn(async move {
        while let Ok(announce) = brx.recv().await {
            send.write_all(&announce).await;
        }
    });

    Ok(())
}
