//! Audit shard request stream.

use crate::Context;
use quinn::{RecvStream, SendStream};
use std::sync::Arc;

/// Send an audit shard request.
pub async fn send<C: Context>(
    send: SendStream,
    recv: RecvStream,
    context: Arc<C>,
) -> anyhow::Result<()> {
    Ok(())
}

/// Receive an audit shard request.
pub async fn recv<C: Context>(
    send: SendStream,
    recv: RecvStream,
    context: Arc<C>,
) -> anyhow::Result<()> {
    Ok(())
}
