//! Work package sharing stream.

use crate::{Context, Network};
use quinn::{RecvStream, SendStream};

/// Send a work package sharing.
pub async fn send<C: Context + Send + Sync + 'static>(
    send: SendStream,
    recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}

/// Receive a work package sharing.
pub async fn recv<C: Context + Send + Sync + 'static>(
    send: SendStream,
    recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}
