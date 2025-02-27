//! Block request stream.

use crate::{Context, Network};
use quinn::{RecvStream, SendStream};

/// Send a block request.
pub async fn send<C: Context + Send + Sync + 'static>(
    send: SendStream,
    recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}

/// Receive a block request.
pub async fn recv<C: Context + Send + Sync + 'static>(
    send: SendStream,
    recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}
