//! Audit announcement stream.

use crate::{Context, Network};
use quinn::{RecvStream, SendStream};

/// Send an audit announcement.
pub async fn send<C: Context + Send + Sync + 'static>(
    send: SendStream,
    recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}

/// Receive an audit announcement.
pub async fn recv<C: Context + Send + Sync + 'static>(
    send: SendStream,
    recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    Ok(())
}
